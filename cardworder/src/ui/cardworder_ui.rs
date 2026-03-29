use core::fmt::Write;

use embedded_fps::FPS;
use embedded_graphics::prelude::WebColors;
use embedded_graphics::primitives::Rectangle;
use embedded_graphics::prelude::{Point, RgbColor};


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

    /// Flush framebuffer content to a custom page address range (for page 2 writes).
    pub fn flush_to_page(&mut self, page_start: u16, page_end: u16) {
        unsafe {
            let screen = &mut self.display.screen;
            screen.dcs().write_command(SetColumnAddress::new(40, 279)).unwrap();
            screen.dcs().write_command(SetPageAddress::new(page_start, page_end)).unwrap();
            screen.dcs().write_command(WriteMemoryStart).unwrap();
            let pixel_data: &[u16] = &self.framebuffer.data.data;
            let bytes: &[u8] = core::slice::from_raw_parts(
                pixel_data.as_ptr() as *const u8,
                pixel_data.len() * 2,
            );
            screen.dcs().di.send_data(DataFormat::U8(bytes)).unwrap();
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

    /// Clear framebuffer without marking pixels dirty.
    /// After drawing, call `mark_prev_extent` to ensure old content is flushed.
    pub fn clear_no_dirty(&mut self, color: Rgb565) {
        self.framebuffer.data.clear_no_dirty(color);
    }

    /// Expand dirty region to include given row range.
    pub fn mark_rows_dirty(&mut self, min_y: usize, max_y: usize) {
        self.framebuffer.data.mark_rows_dirty(min_y, max_y);
    }

    /// Hardware scroll — shifts display content without redrawing framebuffer.
    /// With 90° rotation, this scrolls horizontally on screen.
    /// Only sends a 4-byte SPI command — instant, no pixel data transfer.
    pub fn set_scroll_offset(&mut self, offset: u16) {
        unsafe {
            let screen = &mut self.display.screen;
            screen.dcs().write_command(
                mipidsi::dcs::SetScrollStart::new(offset)
            ).unwrap();
        }
    }

    /// Flush dirty region of framebuffer to display via SPI (partial flush).
    pub fn flip_buffer(&mut self) {
        let t0 = unsafe { esp_idf_svc::sys::esp_timer_get_time() as u64 };
        let fps = self.fps_counter.try_tick_max().unwrap_or(0);
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
                let bytes: &[u8] = core::slice::from_raw_parts(
                    pixel_data[start..end].as_ptr() as *const u8,
                    (end - start) * 2,
                );
                screen.dcs().di.send_data(DataFormat::U8(bytes)).unwrap();
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

    /// Draw text with auto font sizing and word wrap. Returns height used in pixels.
    pub fn draw_text_auto(&mut self, text: &str, color: ThemeColor, x: i32, y: i32, max_width: u32) -> i32 {
        let char_count = text.chars().count();
        let xlarge_max = (max_width / 10) as usize;
        let large_max = (max_width / 9) as usize;
        let med_max = (max_width / 6) as usize;

        if char_count <= xlarge_max {
            self.draw_text_oneline(text, CardFont::XLarge, color, Point::new(x, y), VerticalPosition::Top);
            self.font_height(CardFont::XLarge) as i32
        } else if char_count <= large_max {
            self.draw_text_oneline(text, CardFont::Large, color, Point::new(x, y), VerticalPosition::Top);
            self.font_height(CardFont::Large) as i32
        } else if char_count <= med_max {
            self.draw_text_oneline(text, CardFont::Medium, color, Point::new(x, y), VerticalPosition::Top);
            self.font_height(CardFont::Medium) as i32
        } else {
            // Word wrap with Medium font
            let line_h = self.font_height(CardFont::Medium) as i32;
            let mut cy = y;
            for line in wrap_text(text, med_max) {
                self.draw_text_oneline(line.as_str(), CardFont::Medium, color, Point::new(x, cy), VerticalPosition::Top);
                cy += line_h + 1;
            }
            (cy - y).max(line_h)
        }
    }

    pub fn fill_rect(&mut self, rect: Rectangle, color: Rgb565) {
        self.framebuffer.fill_solid(&rect, color).unwrap();
    }

    pub fn draw_top_line(
        &mut self,
        input_state: &InputState,
        key_event: &Option<(KeyEvent, PressedSymbol)>,
        wifi_connected: bool,
        battery_percent: u8,
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

        // Fn/Shift indicators with lock mode support
        let fn_bg = Rgb565::new(31, 15, 7);    // ~#fd7a39
        let shift_bg = Rgb565::new(7, 15, 21); // ~#3b7aaa
        let mut mod_x = key_descs_x;
        let renderer = &self.renderers[CardFont::Medium as usize];
        let char_w = 6i32; // Medium font width
        let char_h = 10i32;

        // Helper: draw modifier with optional colored background rect
        macro_rules! draw_mod {
            ($label:expr, $show:expr, $locked:expr, $pressed:expr, $bg_color:expr) => {
                if $show || $locked {
                    let label_w = $label.len() as i32 * char_w;
                    if $locked {
                        // Draw colored background rect
                        let rect = Rectangle {
                            top_left: Point::new(mod_x, 0),
                            size: Size::new(label_w as u32 + 2, char_h as u32 + 1),
                        };
                        self.framebuffer.fill_solid(&rect, $bg_color).ok();
                        // Text: white when active, gray when temporarily off
                        let fg = if $pressed { Rgb565::CSS_GRAY } else { Rgb565::WHITE };
                        renderer.render($label, Point::new(mod_x + 1, 1), VerticalPosition::Top,
                            FontColor::Transparent(fg), &mut self.framebuffer).unwrap();
                        mod_x += label_w + 4;
                    } else {
                        // Not locked, just held — white text
                        renderer.render($label, Point::new(mod_x, 1), VerticalPosition::Top,
                            FontColor::Transparent(Rgb565::WHITE), &mut self.framebuffer).unwrap();
                        mod_x += label_w + 2;
                    }
                }
            };
        }

        draw_mod!("fn", input_state.fn_pressed, input_state.fn_locked, input_state.fn_pressed, fn_bg);
        draw_mod!("Aa", input_state.shift_pressed, input_state.shift_locked, input_state.shift_pressed, shift_bg);

        // Other modifiers (Alt, Ctrl, Opt) — simple text
        let mut key_descs = heapless::String::<32>::new();
        for (flag, label) in [
            (input_state.alt_pressed, "alt "),
            (input_state.ctrl_pressed, "ctrl "),
            (input_state.opt_pressed, "opt "),
        ] {
            if flag { let _ = key_descs.push_str(label); }
        }
        if !key_descs.is_empty() {
            let r = renderer.render(key_descs.as_str(), Point::new(mod_x, 1), VerticalPosition::Top,
                FontColor::Transparent(Rgb565::WHITE), &mut self.framebuffer).unwrap();
            mod_x = r.bounding_box.map(|bb| bb.top_left.x + bb.size.width as i32).unwrap_or(mod_x) + 2;
        }

        let pressed_key_desc_x = mod_x;

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

        // Battery percentage left of clock
        let mut bat_text = heapless::String::<8>::new();
        let _ = write!(bat_text, "{}%", battery_percent);
        let bat_color = if battery_percent <= 10 { Rgb565::RED }
                       else if battery_percent <= 30 { Rgb565::YELLOW }
                       else { Rgb565::CSS_LIGHT_GREEN };
        let bat_x = time_x - (bat_text.chars().count() as i32 * 6) - 4;
        self.renderers[CardFont::Medium as usize]
            .render(bat_text.as_str(), Point::new(bat_x, 1), VerticalPosition::Top,
                FontColor::Transparent(bat_color), &mut self.framebuffer)
            .unwrap();

        if wifi_connected {
            self.renderers[CardFont::Icons as usize]
                .render(80 as char, Point::new(bat_x - 11, 0), VerticalPosition::Top,
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

/// Split text at word boundaries to fit within max_chars per line.
fn wrap_text(text: &str, max_chars: usize) -> Vec<String> {
    let mut lines = Vec::new();
    let mut current = String::new();

    for word in text.split_whitespace() {
        let word_len = word.chars().count();
        let cur_len = current.chars().count();

        if cur_len == 0 {
            // First word on line — take it even if too long
            current = word.to_string();
        } else if cur_len + 1 + word_len <= max_chars {
            current.push(' ');
            current.push_str(word);
        } else {
            lines.push(current);
            current = word.to_string();
        }
    }
    if !current.is_empty() {
        lines.push(current);
    }
    if lines.is_empty() {
        lines.push(text.to_string());
    }
    lines
}
