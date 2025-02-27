use std::str;

use display_interface::DisplayError;
use embedded_fps::{StdClock, FPS};
use embedded_graphics::geometry::AnchorPoint;
use embedded_graphics::mono_font::iso_8859_5::FONT_6X13;
use embedded_graphics::mono_font::iso_8859_5::FONT_6X13_BOLD;
use embedded_graphics::prelude::WebColors;
use embedded_graphics::primitives::{Line, Rectangle};
use embedded_graphics::text::{Alignment, Baseline, TextStyleBuilder};
use embedded_graphics::{
    mono_font::{ascii::FONT_4X6, MonoTextStyle},
    prelude::{Point, RgbColor},
    text::Text,
};
use embedded_text::alignment::HorizontalAlignment;
use embedded_text::style::{HeightMode, TextBoxStyleBuilder};
use embedded_text::TextBox;
use esp_idf_svc::hal::prelude::Peripherals;

use embedded_graphics::{pixelcolor::Rgb565, prelude::*, primitives::PrimitiveStyle};

use u8g2_fonts::types::{FontColor, VerticalPosition};
use u8g2_fonts::{fonts, FontRenderer};

use crate::screen::cardputer_screen::CardputerScreen;

pub struct CardworderUi<'a> {
    screen: CardputerScreen<'a>,
    fps_counter: FPS<45, StdClock>,
    debug_small_text_style: MonoTextStyle<'a, Rgb565>,
    pub show_fps: bool,
}

impl CardworderUi<'_> {
    pub fn build<'b>(screen: CardputerScreen<'b>) -> CardworderUi<'b> {
        let fps_counter = FPS::<45, _>::new(StdClock::default());
        let debug_small_text_style = MonoTextStyle::new(&FONT_4X6, Rgb565::WHITE);
        CardworderUi {
            screen: screen,
            fps_counter: fps_counter,
            debug_small_text_style,
            show_fps: true,
        }
    }

    pub fn clear(&mut self, color: Rgb565) {
        self.screen.clear(color);
    }

    pub fn flip_buffer(&mut self) {
        let fps = self.fps_counter.tick();
        if (self.show_fps) {
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

    pub fn draw_long_text(&mut self,is_bold: bool) {
        let text = "- В мои 27 меня уже ничем не удивить!\n- Тебе 35.\n- Что, блин?!";
        let font1 = FontRenderer::new::<fonts::u8g2_font_4x6_t_cyrillic>();
        let result1 = font1
            .render(
                text,
                Point::new(8, 0),
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

        let font = if is_bold {
            &FONT_6X13_BOLD
        }else {
            &FONT_6X13
        };

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
