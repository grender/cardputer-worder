use core::fmt::Write;
use embedded_graphics::pixelcolor::Rgb565;
use embedded_graphics::prelude::WebColors;
use u8g2_fonts::types::VerticalPosition;

use crate::cardputer_hal::input::keyboard::{InputLanguage, PressedSymbol};
use crate::cardputer_hal::input::keyboard_io::KeyEvent;
use crate::screen::{Screen, Snapshot};
use crate::screens::main_menu::MainMenuScreen;
use crate::types::{Command, Core0Action, Core0Result, Msg, SharedState};
use crate::ui::cardworder_ui::{CardFont, CardworderUi, ThemeColor, TOP_BAR_HEIGHT};
use crate::ui::elements::{UiLineElement, UiLineType};
use crate::ui::render::{compose_scrolled_form, render_visible_lines};

pub struct SystemInfoScreen {
    focused_idx: usize,
    wifi_ip: Option<heapless::String<16>>,
    needs_network_info: bool,
    needs_battery: bool,
    battery_mv: u32,
    battery_percent: u8,
}

impl SystemInfoScreen {
    pub fn new() -> Self {
        Self {
            focused_idx: 0,
            wifi_ip: None,
            needs_network_info: true,
            needs_battery: true,
            battery_mv: 0,
            battery_percent: 0,
        }
    }
}

impl Screen for SystemInfoScreen {
    fn on_mount(&mut self, _shared: &SharedState, _state_tx: &std::sync::mpsc::Sender<Snapshot>) -> Command {
        self.needs_network_info = true;
        self.needs_battery = true;
        Command::None
    }

    fn handle_msg(&mut self, msg: Msg, _shared: &SharedState) -> Command {
        match msg {
            Msg::Core0Result(result) => {
                match result {
                    Core0Result::NetworkInfo { ip } => {
                        self.wifi_ip = if ip.is_empty() { None } else { Some(ip) };
                        self.needs_network_info = false;
                    }
                    Core0Result::BatteryReading { mv, percent } => {
                        self.battery_mv = mv;
                        self.battery_percent = percent;
                        self.needs_battery = false;
                    }
                    _ => {}
                }
                Command::None
            }
            Msg::Key(key_msg) => match key_msg.pressed {
                Some((KeyEvent::Pressed, PressedSymbol::Esc))
                | Some((KeyEvent::Pressed, PressedSymbol::Enter)) => {
                    Command::SwitchTo(Box::new(MainMenuScreen::default()))
                }
                Some((KeyEvent::Pressed, PressedSymbol::ArrowDown)) => {
                    self.focused_idx += 1;
                    Command::None
                }
                Some((KeyEvent::Pressed, PressedSymbol::ArrowUp)) => {
                    if self.focused_idx > 0 { self.focused_idx -= 1; }
                    Command::None
                }
                _ => Command::None,
            },
            _ => Command::None,
        }
    }

    fn snapshot(&self, shared: &SharedState) -> Snapshot {
        let free_heap = unsafe { esp_idf_sys::heap_caps_get_free_size(esp_idf_sys::MALLOC_CAP_DEFAULT) };
        let total_heap = unsafe { esp_idf_sys::heap_caps_get_total_size(esp_idf_sys::MALLOC_CAP_DEFAULT) };
        let free_dma = unsafe { esp_idf_sys::heap_caps_get_free_size(esp_idf_sys::MALLOC_CAP_DMA | esp_idf_sys::MALLOC_CAP_INTERNAL) };
        let largest_block = unsafe { esp_idf_sys::heap_caps_get_largest_free_block(esp_idf_sys::MALLOC_CAP_DEFAULT) };
        let uptime_us = unsafe { esp_idf_sys::esp_timer_get_time() as u64 };

        let pending_action = if self.needs_network_info {
            Some(Core0Action::GetNetworkInfo)
        } else if self.needs_battery {
            Some(Core0Action::ReadBattery)
        } else {
            None
        };

        Snapshot::SystemInfo(SystemInfoSnapshot {
            free_heap_bytes: free_heap as u32,
            total_heap_bytes: total_heap as u32,
            free_dma_bytes: free_dma as u32,
            largest_block_bytes: largest_block as u32,
            uptime_secs: (uptime_us / 1_000_000) as u32,
            wifi_connected: shared.wifi_connected,
            wifi_ssid: shared.wifi_ssid.clone(),
            wifi_ip: self.wifi_ip.clone(),
            battery_mv: self.battery_mv,
            battery_percent: self.battery_percent,
            focused_idx: self.focused_idx,
            lang: shared.lang,
            pending_action,
        })
    }
}

pub struct SystemInfoSnapshot {
    pub free_heap_bytes: u32,
    pub total_heap_bytes: u32,
    pub free_dma_bytes: u32,
    pub largest_block_bytes: u32,
    pub uptime_secs: u32,
    pub wifi_connected: bool,
    pub wifi_ssid: Option<heapless::String<32>>,
    pub wifi_ip: Option<heapless::String<16>>,
    pub battery_mv: u32,
    pub battery_percent: u8,
    pub focused_idx: usize,
    pub lang: InputLanguage,
    pub pending_action: Option<Core0Action>,
}

fn t(en: &'static str, ru: &'static str, lang: InputLanguage) -> &'static str {
    match lang { InputLanguage::En => en, InputLanguage::Ru => ru }
}

impl SystemInfoSnapshot {
    pub fn draw(&self, ui: &mut CardworderUi) {
        let font = CardFont::Medium;
        let color = ThemeColor::Text;
        let label_color = ThemeColor::Color(Rgb565::CSS_GRAY);
        let l = self.lang;

        const SCREEN_HEIGHT: u32 = 135;
        let viewport_height = SCREEN_HEIGHT - TOP_BAR_HEIGHT;

        let used = self.total_heap_bytes.saturating_sub(self.free_heap_bytes);
        let pct = if self.total_heap_bytes > 0 { (used as u64 * 100 / self.total_heap_bytes as u64) as u32 } else { 0 };
        let hours = self.uptime_secs / 3600;
        let mins = (self.uptime_secs % 3600) / 60;
        let secs = self.uptime_secs % 60;

        let mut s_heap = heapless::String::<48>::new();
        let _ = write!(s_heap, "{}: {}KB / {}KB", t("Heap", "Куча", l), self.free_heap_bytes / 1024, self.total_heap_bytes / 1024);
        let mut s_dma = heapless::String::<48>::new();
        let _ = write!(s_dma, "{}: {}KB", t("DMA free", "DMA свобод.", l), self.free_dma_bytes / 1024);
        let mut s_block = heapless::String::<48>::new();
        let _ = write!(s_block, "{}: {}KB", t("Largest block", "Макс. блок", l), self.largest_block_bytes / 1024);
        let mut s_uptime = heapless::String::<48>::new();
        let _ = write!(s_uptime, "{}: {:02}:{:02}:{:02}", t("Uptime", "Время работы", l), hours, mins, secs);
        let mut s_pct = heapless::String::<48>::new();
        let _ = write!(s_pct, "{}: {}%", t("Heap used", "Куча занята", l), pct);

        let mut s_bat = heapless::String::<48>::new();
        let _ = write!(s_bat, "{}: {} mV ({}%)", t("Battery", "Батарея", l), self.battery_mv, self.battery_percent);

        let wifi_status = if self.wifi_connected { t("WiFi: Connected", "WiFi: Подключен", l) } else { t("WiFi: Not connected", "WiFi: Не подключен", l) };
        let mut s_ssid = heapless::String::<48>::new();
        if let Some(ref ssid) = self.wifi_ssid {
            let _ = write!(s_ssid, "SSID: {}", ssid.as_str());
        }
        let mut s_ip = heapless::String::<48>::new();
        if let Some(ref ip) = self.wifi_ip {
            let _ = write!(s_ip, "IP: {}", ip.as_str());
        }

        let wifi_color = if self.wifi_connected { ThemeColor::Color(Rgb565::new(0, 50, 0)) } else { label_color };

        let mut lines: Vec<UiLineType> = vec![
            UiLineType::Elements(vec![UiLineElement::Text(t("System Info", "Системная информация", l), CardFont::Large, VerticalPosition::Top, ThemeColor::Selected)]),
            UiLineType::Elements(vec![UiLineElement::Text(s_heap.as_str(), font, VerticalPosition::Top, color)]),
            UiLineType::Elements(vec![UiLineElement::Text(s_dma.as_str(), font, VerticalPosition::Top, color)]),
            UiLineType::Elements(vec![UiLineElement::Text(s_block.as_str(), font, VerticalPosition::Top, color)]),
            UiLineType::Elements(vec![UiLineElement::Text(s_uptime.as_str(), font, VerticalPosition::Top, color)]),
            UiLineType::Elements(vec![UiLineElement::Text(s_pct.as_str(), font, VerticalPosition::Top, label_color)]),
            UiLineType::Spacer(4),
            UiLineType::Elements(vec![UiLineElement::Text(s_bat.as_str(), font, VerticalPosition::Top, {
                if self.battery_percent <= 10 { ThemeColor::Error }
                else if self.battery_percent <= 30 { ThemeColor::Color(Rgb565::new(31, 50, 0)) }
                else { ThemeColor::Color(Rgb565::new(0, 50, 0)) }
            })]),
            UiLineType::Spacer(4),
            UiLineType::Elements(vec![UiLineElement::Text(wifi_status, font, VerticalPosition::Top, wifi_color)]),
        ];
        if self.wifi_connected {
            if !s_ssid.is_empty() {
                lines.push(UiLineType::Elements(vec![UiLineElement::Text(s_ssid.as_str(), font, VerticalPosition::Top, color)]));
            }
            if !s_ip.is_empty() {
                lines.push(UiLineType::Elements(vec![UiLineElement::Text(s_ip.as_str(), font, VerticalPosition::Top, color)]));
            }
        }
        lines.push(UiLineType::Spacer(4));
        lines.push(UiLineType::Elements(vec![UiLineElement::Text(t("<- Back (Enter/Esc)", "<- Назад (Enter/Esc)", l), font, VerticalPosition::Top, ThemeColor::Selected)]));

        let composed = compose_scrolled_form(&lines, self.focused_idx.min(lines.len().saturating_sub(1)), viewport_height, TOP_BAR_HEIGHT as i32, ui);
        render_visible_lines(&composed, &lines, ui);
    }
}
