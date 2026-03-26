use core::fmt::Write;
use embedded_graphics::pixelcolor::Rgb565;
use embedded_graphics::prelude::{Point, RgbColor, WebColors};
use u8g2_fonts::types::VerticalPosition;

use crate::cardputer_hal::cardputer_hal::CardputerHal;
use crate::cardputer_hal::input::keyboard::PressedSymbol;
use crate::cardputer_hal::input::keyboard_io::KeyEvent;
use crate::screen::{Renderable, Screen};
use crate::screens::main_menu::MainMenuScreen;
use crate::types::{Command, Msg, SharedState};
use crate::ui::cardworder_ui::{CardFont, CardworderUi, ThemeColor, TOP_BAR_HEIGHT};

pub struct SystemInfoScreen {}

impl SystemInfoScreen {
    pub fn new() -> Self {
        Self {}
    }
}

impl Screen for SystemInfoScreen {
    fn handle_msg(
        &mut self,
        msg: Msg,
        _hal: &mut CardputerHal<'_>,
        _shared: &SharedState,
    ) -> Command {
        match msg {
            Msg::Key(key_msg) => match key_msg.pressed {
                Some((KeyEvent::Pressed, PressedSymbol::Esc))
                | Some((KeyEvent::Pressed, PressedSymbol::Enter)) => {
                    Command::SwitchTo(Box::new(MainMenuScreen::default()))
                }
                _ => Command::None,
            },
            _ => Command::None,
        }
    }

    fn snapshot(&self) -> Box<dyn Renderable> {
        let free_heap = unsafe {
            esp_idf_sys::heap_caps_get_free_size(esp_idf_sys::MALLOC_CAP_DEFAULT)
        };
        let total_heap = unsafe {
            esp_idf_sys::heap_caps_get_total_size(esp_idf_sys::MALLOC_CAP_DEFAULT)
        };
        let free_dma = unsafe {
            esp_idf_sys::heap_caps_get_free_size(
                esp_idf_sys::MALLOC_CAP_DMA | esp_idf_sys::MALLOC_CAP_INTERNAL,
            )
        };
        let largest_block = unsafe {
            esp_idf_sys::heap_caps_get_largest_free_block(esp_idf_sys::MALLOC_CAP_DEFAULT)
        };
        let uptime_us = unsafe { esp_idf_sys::esp_timer_get_time() as u64 };

        Box::new(SystemInfoSnapshot {
            free_heap_bytes: free_heap as u32,
            total_heap_bytes: total_heap as u32,
            free_dma_bytes: free_dma as u32,
            largest_block_bytes: largest_block as u32,
            uptime_secs: (uptime_us / 1_000_000) as u32,
        })
    }
}

struct SystemInfoSnapshot {
    free_heap_bytes: u32,
    total_heap_bytes: u32,
    free_dma_bytes: u32,
    largest_block_bytes: u32,
    uptime_secs: u32,
}

unsafe impl Send for SystemInfoSnapshot {}

impl Renderable for SystemInfoSnapshot {
    fn draw(&self, ui: &mut CardworderUi) {
        let font = CardFont::Medium;
        let color = ThemeColor::Text;
        let label_color = ThemeColor::Color(Rgb565::CSS_GRAY);
        let line_h = ui.font_height(font) as i32 + 2;
        let mut y = TOP_BAR_HEIGHT as i32 + 4;

        // Title
        ui.draw_text_oneline(
            "System Info",
            CardFont::Large,
            ThemeColor::Selected,
            Point::new(4, y),
            VerticalPosition::Top,
        );
        y += ui.font_height(CardFont::Large) as i32 + 6;

        // Free heap
        let used = self.total_heap_bytes.saturating_sub(self.free_heap_bytes);
        let mut text = heapless::String::<48>::new();
        let _ = write!(text, "Heap: {}KB / {}KB", self.free_heap_bytes / 1024, self.total_heap_bytes / 1024);
        ui.draw_text_oneline(text.as_str(), font, color, Point::new(4, y), VerticalPosition::Top);
        y += line_h;

        // Free DMA
        text.clear();
        let _ = write!(text, "DMA free: {}KB", self.free_dma_bytes / 1024);
        ui.draw_text_oneline(text.as_str(), font, color, Point::new(4, y), VerticalPosition::Top);
        y += line_h;

        // Largest free block
        text.clear();
        let _ = write!(text, "Largest block: {}KB", self.largest_block_bytes / 1024);
        ui.draw_text_oneline(text.as_str(), font, color, Point::new(4, y), VerticalPosition::Top);
        y += line_h;

        // Uptime
        let hours = self.uptime_secs / 3600;
        let mins = (self.uptime_secs % 3600) / 60;
        let secs = self.uptime_secs % 60;
        text.clear();
        let _ = write!(text, "Uptime: {:02}:{:02}:{:02}", hours, mins, secs);
        ui.draw_text_oneline(text.as_str(), font, color, Point::new(4, y), VerticalPosition::Top);
        y += line_h;

        // Heap usage percentage
        let pct = if self.total_heap_bytes > 0 {
            (used as u64 * 100 / self.total_heap_bytes as u64) as u32
        } else {
            0
        };
        text.clear();
        let _ = write!(text, "Heap used: {}%", pct);
        ui.draw_text_oneline(text.as_str(), font, label_color, Point::new(4, y), VerticalPosition::Top);
        y += line_h + 4;

        // Back button
        ui.draw_text_oneline(
            "<- Back (Enter/Esc)",
            font,
            ThemeColor::Selected,
            Point::new(4, y),
            VerticalPosition::Top,
        );
    }
}
