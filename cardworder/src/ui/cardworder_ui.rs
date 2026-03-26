use core::fmt::Write;

use embedded_fps::FPS;
use embedded_graphics::mono_font::iso_8859_5::FONT_6X13;
use embedded_graphics::mono_font::iso_8859_5::FONT_6X13_BOLD;
use embedded_graphics::prelude::WebColors;
use embedded_graphics::primitives::Rectangle;
use embedded_graphics::{
    mono_font::MonoTextStyle,
    prelude::{Point, RgbColor},
};

use embedded_text::alignment::HorizontalAlignment;
use embedded_text::style::{HeightMode, TextBoxStyleBuilder};
use embedded_text::TextBox;

use embedded_graphics::{pixelcolor::Rgb565, prelude::*};

use display_interface::DataFormat;
use display_interface::WriteOnlyDataCommand;
use embedded_graphics_framebuf::FrameBuf;
use embedded_time::rate::Fraction;
use esp_idf_sys::{localtime_r, time, time_t, tm};
use mipidsi::dcs::{SetColumnAddress, SetPageAddress, WriteMemoryStart};
use u8g2_fonts::types::{FontColor, VerticalPosition};
use u8g2_fonts::Content;
use u8g2_fonts::{fonts, FontRenderer};

use crate::cardputer_hal::input::keyboard::InputState;
use crate::cardputer_hal::input::keyboard::PressedSymbol;
use crate::cardputer_hal::input::keyboard_io::KeyEvent;
use crate::cardputer_hal::screen::display::CardputerDisplay;
use crate::cardputer_hal::screen::framebuffer::CardputerFramebuffer;

/// Height of the top bar (status line + separator) in pixels.
pub const TOP_BAR_HEIGHT: u32 = 12;

pub struct CardworderClock {}

const FONT_COUNT: usize = 7;

pub struct CardworderUi {
    framebuffer: FrameBuf<Rgb565, CardputerFramebuffer>,
    display: &'static mut CardputerDisplay<'static>,
    fps_counter: FPS<45, CardworderClock>,
    renderers: [FontRenderer; FONT_COUNT],
    pub show_fps: bool,
    // Perf stats
    flip_sample_count: u64,
    flip_total_us_acc: u64,
    flip_fps_overlay_us_acc: u64,
    flip_flush_us_acc: u64,
}

#[derive(Copy, Clone)]
pub enum ThemeColor {
    Background,
    Text,
    Selected,
    Error,
    Color(Rgb565),
}

#[derive(Eq, PartialEq, Hash, Copy, Clone)]
#[repr(usize)]
pub enum CardFont {
    XSmall,
    Small,
    Medium,
    Large,
    XLarge,
    Icons,
    IconsHuge,
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
    ) -> embedded_time::Timer<'_,
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

fn get_rgb565(color: ThemeColor) -> Rgb565 {
    match color {
        ThemeColor::Background => Rgb565::BLACK,
        ThemeColor::Text => Rgb565::WHITE,
        ThemeColor::Selected => Rgb565::CSS_LIGHT_BLUE,
        ThemeColor::Error => Rgb565::RED,
        ThemeColor::Color(c) => c,
    }
}

impl CardworderUi {
    pub fn build(
        framebuffer: FrameBuf<Rgb565, CardputerFramebuffer>,
        display: &'static mut CardputerDisplay<'static>,
    ) -> CardworderUi {
        let fps_counter = FPS::<45, _>::new(CardworderClock::default());
        let renderers = [
            FontRenderer::new::<fonts::u8g2_font_4x6_t_cyrillic>(),      // XSmall = 0
            FontRenderer::new::<fonts::u8g2_font_5x8_t_cyrillic>(),      // Small = 1
            FontRenderer::new::<fonts::u8g2_font_6x12_t_cyrillic>(),     // Medium = 2
            FontRenderer::new::<fonts::u8g2_font_9x15_t_cyrillic>(),     // Large = 3
            FontRenderer::new::<fonts::u8g2_font_10x20_t_cyrillic>(),    // XLarge = 4
            FontRenderer::new::<fonts::u8g2_font_open_iconic_embedded_1x_t>(), // Icons = 5
            FontRenderer::new::<fonts::u8g2_font_streamline_all_t>(),    // IconsHuge = 6
        ];
        CardworderUi {
            framebuffer,
            display,
            fps_counter,
            renderers,
            show_fps: false,
            flip_sample_count: 0,
            flip_total_us_acc: 0,
            flip_fps_overlay_us_acc: 0,
            flip_flush_us_acc: 0,
        }
    }

    pub fn font_height(&self, font: CardFont) -> u32 {
        self.renderers[font as usize].get_default_line_height() as u32
    }

    pub fn font_width(&self, font: CardFont) -> u32 {
        self.renderers[font as usize]
            .get_glyph_bounding_box(VerticalPosition::Top).size.width as u32
    }

    pub fn clear(&mut self, color: Rgb565) {
        self.framebuffer.clear(color).ok();
    }

    /// Flush dirty region of framebuffer to display via SPI (partial flush).
    pub fn flip_buffer(&mut self) {
        let t0 = unsafe { esp_idf_svc::sys::esp_timer_get_time() as u64 };
        let fps = self.fps_counter.tick();
        if self.show_fps {
            let mut fps_text = heapless::String::<16>::new();
            let _ = write!(fps_text, "FPS: {}", fps);
            let renderer = &self.renderers[CardFont::XSmall as usize];
            let glyph_size = renderer.get_glyph_bounding_box(VerticalPosition::Top).size;
            let area = Rectangle {
                top_left: Point::new(0, 0),
                size: Size::new(
                    glyph_size.width as u32 * fps_text.len() as u32 + 2,
                    glyph_size.height as u32 + 2,
                ),
            };
            self.framebuffer.fill_solid(&area, get_rgb565(ThemeColor::Background)).ok();
            renderer.render(
                fps_text.as_str(), Point::new(1, 1), VerticalPosition::Top,
                FontColor::Transparent(get_rgb565(ThemeColor::Text)), &mut self.framebuffer,
            ).unwrap();
        }
        let t_after_overlay = unsafe { esp_idf_svc::sys::esp_timer_get_time() as u64 };

        // Direct partial SPI flush — only send dirty rows
        if let Some((_min_x, min_y, _max_x, max_y)) = self.framebuffer.data.take_dirty_bbox() {
            unsafe {
                let screen = &mut self.display.screen;
                screen.dcs().write_command(SetColumnAddress::new(40, 279)).unwrap();
                screen.dcs().write_command(SetPageAddress::new(
                    53 + min_y as u16, 53 + max_y as u16,
                )).unwrap();
                screen.dcs().write_command(WriteMemoryStart).unwrap();
                let start = min_y * 240;
                let end = (max_y + 1) * 240;
                let pixel_data: &[u16] = &self.framebuffer.data.data;
                screen.dcs().di.send_data(DataFormat::U16(&pixel_data[start..end])).unwrap();
            }
        }

        let t_after_flush = unsafe { esp_idf_svc::sys::esp_timer_get_time() as u64 };
        let total_us = t_after_flush - t0;
        let fps_overlay_us = t_after_overlay - t0;
        let flush_us = t_after_flush - t_after_overlay;

        self.flip_sample_count = self.flip_sample_count.wrapping_add(1);
        self.flip_total_us_acc += total_us;
        self.flip_fps_overlay_us_acc += fps_overlay_us;
        self.flip_flush_us_acc += flush_us;

        if self.flip_sample_count >= 30 {
            let n = self.flip_sample_count.max(1);
            log::info!(
                "perf flip_buffer avg us (total={}, fps_overlay={}, flush={})",
                self.flip_total_us_acc / n, self.flip_fps_overlay_us_acc / n, self.flip_flush_us_acc / n,
            );
            self.flip_sample_count = 0;
            self.flip_total_us_acc = 0;
            self.flip_fps_overlay_us_acc = 0;
            self.flip_flush_us_acc = 0;
        }
    }

    pub fn draw_starting_line_text(&mut self, text: &str, bg_color: Rgb565, font_color: Rgb565) {
        let renderer = &self.renderers[CardFont::Medium as usize];
        let font_height = renderer.get_default_line_height();
        let top_line_area = Rectangle {
            top_left: Point { x: 0, y: 135 - font_height as i32 },
            size: Size { width: 240, height: font_height },
        };
        self.framebuffer.fill_solid(&top_line_area, bg_color).unwrap();
        renderer.render(
            text, Point::new(0, 135 - font_height as i32), VerticalPosition::Top,
            FontColor::Transparent(font_color), &mut self.framebuffer,
        ).unwrap();
    }

    pub fn draw_text_oneline(
        &mut self, s: impl Content, font: CardFont, color: ThemeColor,
        point: Point, vertical_position: VerticalPosition,
    ) -> Option<Rectangle> {
        self.renderers[font as usize]
            .render(s, point, vertical_position,
                FontColor::Transparent(get_rgb565(color)), &mut self.framebuffer)
            .unwrap().bounding_box
    }

    pub fn fill_rect(&mut self, rect: Rectangle, color: Rgb565) {
        self.framebuffer.fill_solid(&rect, color).unwrap();
    }

    pub fn draw_top_line(
        &mut self,
        input_state: &InputState,
        key_event: &Option<(KeyEvent, PressedSymbol)>,
        wifi_connected: bool,
    ) {
        let top_line_area = Rectangle {
            top_left: Point { x: 0, y: 0 },
            size: Size { width: 240, height: 10 },
        };
        self.framebuffer.fill_solid(&top_line_area, Rgb565::BLACK).unwrap();

        let top_line_area_separator = Rectangle {
            top_left: Point { x: 0, y: 11 },
            size: Size { width: 240, height: 1 },
        };
        self.framebuffer.fill_solid(&top_line_area_separator, Rgb565::CSS_GRAY).unwrap();

        let (lang_text, lang_color) = match input_state.lang {
            crate::cardputer_hal::input::keyboard::InputLanguage::En => ("ENG", Rgb565::BLUE),
            crate::cardputer_hal::input::keyboard::InputLanguage::Ru => ("РУС", Rgb565::RED),
        };
        let lang_shower_rect = &self.renderers[CardFont::Medium as usize]
            .render(lang_text, Point::new(1, 1), VerticalPosition::Top,
                FontColor::Transparent(lang_color), &mut self.framebuffer)
            .unwrap();

        let separator_x = lang_shower_rect.bounding_box
            .map(|bb| bb.top_left.x + bb.size.width as i32).unwrap_or(20) + 2;
        let separator_rect = Rectangle {
            top_left: Point { x: separator_x, y: 1 },
            size: Size { width: 2, height: 8 },
        };
        let key_descs_x = separator_x + separator_rect.size.width as i32 + 2;
        self.framebuffer.fill_solid(&separator_rect, Rgb565::CSS_GRAY).unwrap();

        let mut key_descs = heapless::String::<32>::new();
        for (flag, label) in [
            (input_state.fn_pressed, "Fn "),
            (input_state.shift_pressed, "Shft "),
            (input_state.alt_pressed, "Alt "),
            (input_state.ctrl_pressed, "Ctrl "),
            (input_state.opt_pressed, "Opt "),
        ] {
            if flag { let _ = key_descs.push_str(label); }
        }

        let key_descs_rect = &self.renderers[CardFont::Medium as usize]
            .render(key_descs.as_str(), Point::new(key_descs_x, 1), VerticalPosition::Top,
                FontColor::Transparent(Rgb565::WHITE), &mut self.framebuffer)
            .unwrap();

        let pressed_key_desc_x = key_descs_rect.bounding_box
            .map(|bb| bb.top_left.x + bb.size.width as i32).unwrap_or(key_descs_x + 20) + 2;

        if let Some((ke, PressedSymbol::Char(c))) = key_event {
            let mut buf = [0u8; 4];
            let char_print = c.encode_utf8(&mut buf);
            let ke_print = match ke { KeyEvent::Pressed => "P ", KeyEvent::Released => "R " };
            let mut desc = heapless::String::<16>::new();
            let _ = write!(desc, "{}{}", ke_print, char_print);
            &self.renderers[CardFont::Medium as usize]
                .render(desc.as_str(), Point::new(pressed_key_desc_x, 1), VerticalPosition::Top,
                    FontColor::Transparent(Rgb565::CSS_DARK_GRAY), &mut self.framebuffer)
                .unwrap();
        }

        let time_x = 240 - 2 - 8 * 6; // 8 chars "HH:MM:SS" × 6px Medium font width
        if wifi_connected {
            self.renderers[CardFont::Icons as usize]
                .render(80 as char, Point::new(time_x - 11, 0), VerticalPosition::Top,
                    FontColor::Transparent(Rgb565::CSS_LIGHT_GREEN), &mut self.framebuffer)
                .unwrap();
        }

        let mut tm = tm { tm_sec: 0, tm_min: 0, tm_hour: 0, tm_mday: 0, tm_mon: 0, tm_year: 0, tm_wday: 0, tm_yday: 0, tm_isdst: 0 };
        let mut now_time: time_t = 0;
        unsafe { time(&mut now_time); localtime_r(&now_time, &mut tm); }
        let mut formatted = heapless::String::<16>::new();
        let _ = write!(formatted, "{:02}:{:02}:{:02}", tm.tm_hour, tm.tm_min, tm.tm_sec);
        &self.renderers[CardFont::Medium as usize]
            .render(formatted.as_str(), Point::new(time_x, 1), VerticalPosition::Top,
                FontColor::Transparent(Rgb565::WHITE), &mut self.framebuffer)
            .unwrap();
    }

    pub fn draw_text_large(&mut self, text: &str, x: i32, y: i32, font_color: Rgb565) {
        &self.renderers[CardFont::Large as usize]
            .render(text, Point::new(x, y), VerticalPosition::Top,
                FontColor::Transparent(font_color), &mut self.framebuffer)
            .unwrap();
    }
}
