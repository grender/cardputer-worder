use embedded_fps::FPS;
use embedded_graphics::geometry::AnchorPoint;
use embedded_graphics::mono_font::iso_8859_5::FONT_6X13;
use embedded_graphics::mono_font::iso_8859_5::FONT_6X13_BOLD;
use embedded_graphics::prelude::WebColors;
use embedded_graphics::primitives::Rectangle;
use embedded_graphics::text::{Alignment, Baseline, TextStyleBuilder};
use embedded_graphics::{
    mono_font::{ascii::FONT_4X6, MonoTextStyle},
    prelude::{Point, RgbColor},
    text::Text,
};

use embedded_text::alignment::HorizontalAlignment;
use embedded_text::style::{HeightMode, TextBoxStyleBuilder};
use embedded_text::TextBox;

use embedded_graphics::{pixelcolor::Rgb565, prelude::*};

use embedded_time::rate::Fraction;
use esp_idf_sys::{localtime_r, time, time_t, tm};
use u8g2_fonts::types::{FontColor, VerticalPosition};
use u8g2_fonts::{fonts, FontRenderer};

use crate::cardputer_hal::input::keyboard::InputLanguage;
use crate::cardputer_hal::input::keyboard::InputState;
use crate::cardputer_hal::input::keyboard::PressedSymbol;
use crate::cardputer_hal::input::keyboard_io::KeyEvent;
use crate::cardputer_hal::screen::cardputer_screen::CardputerScreen;

pub struct CardworderClock {}
pub struct CardworderUi<'a> {
    screen: CardputerScreen<'a>,
    fps_counter: FPS<45, CardworderClock>,
    debug_small_text_style: MonoTextStyle<'a, Rgb565>,
    pub show_fps: bool,
}

impl Default for CardworderClock {
    fn default() -> Self {
        CardworderClock {}
    }
}

impl embedded_time::clock::Clock for CardworderClock {
    type T = u64;
    const SCALING_FACTOR: Fraction = Fraction::new(1, 1000000);

    fn try_now(&self) -> Result<embedded_time::Instant<Self>, embedded_time::clock::Error> {
        let now = unsafe { esp_idf_svc::sys::esp_timer_get_time() };
        Ok(embedded_time::Instant::<Self>::new(now as u64))
    }

    fn new_timer<Dur: embedded_time::duration::Duration>(
        &self,
        duration: Dur,
    ) -> embedded_time::Timer<
        embedded_time::timer::param::OneShot,
        embedded_time::timer::param::Armed,
        Self,
        Dur,
    >
    where
        Dur: embedded_time::fixed_point::FixedPoint,
    {
        embedded_time::Timer::<
            embedded_time::timer::param::None,
            embedded_time::timer::param::None,
            Self,
            Dur,
        >::new(&self, duration)
    }
}

impl CardworderUi<'_> {
    pub fn build<'b>(screen: CardputerScreen<'b>) -> CardworderUi<'b> {
        let fps_counter = FPS::<45, _>::new(CardworderClock::default());
        let debug_small_text_style = MonoTextStyle::new(&FONT_4X6, Rgb565::WHITE);
        CardworderUi {
            screen: screen,
            fps_counter: fps_counter,
            debug_small_text_style,
            show_fps: false,
        }
    }

    pub fn clear(&mut self, color: Rgb565) {
        self.screen.clear(color);
    }

    pub fn flip_buffer(&mut self) {
        let fps = self.fps_counter.tick();
        if self.show_fps {
            let fps_text = format!("FPS: {}", fps);
            let text_style = TextStyleBuilder::new()
                .baseline(Baseline::Top)
                .alignment(Alignment::Left)
                .build();
            let text = Text::with_text_style(
                &fps_text,
                Point::new(0, 0),
                self.debug_small_text_style,
                text_style,
            );

            let text_box = text.bounding_box();

            let mut new_size = text_box.size.clone();
            new_size.height += 1;
            new_size.width += 1;
            let fill_box = text_box.resized(new_size, AnchorPoint::TopLeft);
            self.screen
                .fill_solid(&fill_box, Rgb565::CSS_BLACK)
                .unwrap();

            text.draw(&mut self.screen).unwrap();
        }
        self.screen.flush_framebuffer();
    }

    pub fn backlight_off(&mut self) {
        self.screen.backlight_off();
    }

    pub fn backlight_on(&mut self) {
        self.screen.backlight_on();
    }

    pub fn draw_starting_line(&mut self, text: &str, bg_color: Rgb565, font_color: Rgb565) {
        let font1 = FontRenderer::new::<fonts::u8g2_font_6x12_t_cyrillic>();

        let top_line_area = Rectangle {
            top_left: Point {
                x: 0,
                y: 135 - font1.get_default_line_height() as i32,
            },
            size: Size {
                width: 240,
                height: font1.get_default_line_height(),
            },
        };
        self.screen.fill_solid(&top_line_area, bg_color).unwrap();

        font1
            .render(
                text,
                Point::new(0, 135 - font1.get_default_line_height() as i32),
                VerticalPosition::Top,
                FontColor::Transparent(font_color),
                &mut self.screen,
            )
            .unwrap();
        
    }

    pub fn draw_top_line(
        &mut self,
        input_state: &InputState,
        key_event: &Option<(KeyEvent, PressedSymbol)>,
    ) {
        let top_line_area = Rectangle {
            top_left: Point { x: 0, y: 0 },
            size: Size {
                width: 240,
                height: 8,
            },
        };
        self.screen
            .fill_solid(&top_line_area, Rgb565::BLACK)
            .unwrap();

        let top_line_area_separator = Rectangle {
            top_left: Point { x: 0, y: 9 },
            size: Size {
                width: 240,
                height: 1,
            },
        };
        self.screen
            .fill_solid(&top_line_area_separator, Rgb565::CSS_GRAY)
            .unwrap();

        let font1 = FontRenderer::new::<fonts::u8g2_font_4x6_t_cyrillic>();
        let (lang_text, lang_color) = match input_state.lang {
            InputLanguage::En => ("ENG", Rgb565::BLUE),
            InputLanguage::Ru => ("РУС", Rgb565::RED),
        };
        let lang_shower_rect = font1
            .render(
                lang_text,
                Point::new(1, 1),
                VerticalPosition::Top,
                FontColor::Transparent(lang_color),
                &mut self.screen,
            )
            .unwrap();

        let separator_x = lang_shower_rect
            .bounding_box
            .map(|bb| bb.top_left.x + bb.size.width as i32)
            .unwrap_or(20)
            + 2;

        let separator_rect = Rectangle {
            top_left: Point {
                x: separator_x,
                y: 1,
            },
            size: Size {
                width: 2,
                height: 6,
            },
        };

        let key_descs_x = separator_x + separator_rect.size.width as i32 + 2;

        self.screen
            .fill_solid(&separator_rect, Rgb565::CSS_GRAY)
            .unwrap();

        let key_descs: String = [
            (input_state.fn_pressed, "Fn "),
            (input_state.shift_pressed, "Shft "),
            (input_state.alt_pressed, "Alt "),
            (input_state.ctrl_pressed, "Ctrl "),
            (input_state.opt_pressed, "Opt "),
        ]
        .iter()
        .filter(|(flag, _)| *flag)
        .map(|(_, value)| *value)
        .collect();

        let key_descs_rect = font1
            .render(
                key_descs.as_str(),
                Point::new(key_descs_x, 1),
                VerticalPosition::Top,
                FontColor::Transparent(Rgb565::WHITE),
                &mut self.screen,
            )
            .unwrap();

        let pressed_key_desc_x = key_descs_rect
            .bounding_box
            .map(|bb| bb.top_left.x + bb.size.width as i32)
            .unwrap_or(key_descs_x + 20)
            + 2;

        match key_event {
            Some((ke, PressedSymbol::Char(c))) => {
                let mut pressed_str_buf = [0u8; 4];
                let char_print = c.encode_utf8(&mut pressed_str_buf);
                let ke_print = match ke {
                    KeyEvent::Pressed => "P ",
                    KeyEvent::Released => "R ",
                };
                let pressed_key_desc = format!("{}{}", ke_print, char_print);
                font1
                    .render(
                        pressed_key_desc.as_str(),
                        Point::new(pressed_key_desc_x, 1),
                        VerticalPosition::Top,
                        FontColor::Transparent(Rgb565::CSS_DARK_GRAY),
                        &mut self.screen,
                    )
                    .unwrap();
            }
            _ => {}
        };


        let fontIcon = FontRenderer::new::<fonts::u8g2_font_open_iconic_embedded_1x_t>();
        fontIcon.render(80 as char, Point::new(240 - 1 - 4 * 8 - 9, 0), VerticalPosition::Top, FontColor::Transparent(Rgb565::WHITE), &mut self.screen).unwrap();

        let time_x = 240 - 1 - 4 * 8;

        let mut tm = tm {
            tm_sec: 0,
            tm_min: 0,
            tm_hour: 0,
            tm_mday: 0,
            tm_mon: 0,
            tm_year: 0,
            tm_wday: 0,
            tm_yday: 0,
            tm_isdst: 0,
        };
        let mut now_time: time_t = 0;
        unsafe {
            // TODO: move to a separate file
            time(&mut now_time);
            localtime_r(&now_time, &mut tm);
        }

        let formatted = format!("{:02}:{:02}:{:02}", tm.tm_hour, tm.tm_min, tm.tm_sec);

        font1
            .render(
                formatted.as_str(),
                Point::new(time_x, 1),
                VerticalPosition::Top,
                FontColor::Transparent(Rgb565::WHITE),
                &mut self.screen,
            )
            .unwrap();
    }


    pub fn draw_text_huge(&mut self, text: &str, x: i32, y: i32, font_color: Rgb565) {
        let font1 = FontRenderer::new::<fonts::u8g2_font_6x12_t_cyrillic>();
        font1.render(text, Point::new(x, y), VerticalPosition::Top, FontColor::Transparent(font_color), &mut self.screen).unwrap();
    }

    pub fn draw_long_text(&mut self, is_bold: bool) {
        let text = "- В мои 27 меня уже ничем не удивить!\n- Тебе 35.\n- Что, блин?!";
        let font1 = FontRenderer::new::<fonts::u8g2_font_4x6_t_cyrillic>();
        let result1 = font1
            .render(
                text,
                Point::new(8, 10),
                VerticalPosition::Top,
                FontColor::Transparent(Rgb565::BLUE),
                &mut self.screen,
            )
            .unwrap();

        let font2 = FontRenderer::new::<fonts::u8g2_font_9x15_t_cyrillic>();
        let result2 = font2
            .render(
                text,
                Point::new(8, result1.bounding_box.unwrap().size.height as i32),
                VerticalPosition::Top,
                FontColor::Transparent(Rgb565::BLUE),
                &mut self.screen,
            )
            .unwrap();

        // let encoded = ISO_8859_5.encode(text, encoding::EncoderTrap::NcrEscape).unwrap();

        let font = if is_bold { &FONT_6X13_BOLD } else { &FONT_6X13 };

        let character_style = MonoTextStyle::new(font, Rgb565::BLUE);
        let textbox_style = TextBoxStyleBuilder::new()
            .height_mode(HeightMode::FitToText)
            .alignment(HorizontalAlignment::Justified)
            .paragraph_spacing(0)
            .build();
        let bounds = Rectangle::new(
            Point::new(
                8,
                result2
                    .bounding_box
                    .unwrap()
                    .anchor_y(embedded_graphics::geometry::AnchorY::Bottom) as i32,
            ),
            Size::new(self.screen.framebuffer.width() as u32 - 16, 0),
        );

        let text_box = TextBox::with_textbox_style(text, bounds, character_style, textbox_style);
        text_box.draw(&mut self.screen).unwrap();
    }
}
