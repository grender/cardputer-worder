use core::fmt::Write;
use embedded_graphics::prelude::WebColors;
use u8g2_fonts::types::VerticalPosition;

use crate::cardputer_hal::input::keyboard::{InputLanguage, PressedSymbol};
use crate::cardputer_hal::input::keyboard_io::KeyEvent;
use crate::cardputer_hal::wifi::wifi::WifiConfig;
use crate::screen::{Screen, Snapshot};
use crate::screens::main_menu::MainMenuScreen;
use crate::screens::wifi_connect::WifiConnectScreen;
use crate::types::{Command, Core0Action, Core0Result, Msg, SharedState, WifiConfigList};
use crate::ui::cardworder_ui::{CardFont, CardworderUi, ThemeColor, TOP_BAR_HEIGHT};
use crate::ui::elements::{UiLineElement, UiLineType};
use crate::ui::render::{compose_scrolled_form, render_visible_lines};

pub struct WifiConfigScreen {
    configs: Vec<WifiConfig>,
    focused_idx: usize,
    loading: bool,
    needs_save: bool,
    wifi_connected: bool,
    disconnecting: bool,
}

impl WifiConfigScreen {
    pub fn new() -> Self {
        Self {
            configs: Vec::new(),
            focused_idx: 0,
            loading: true,
            needs_save: false,
            wifi_connected: false,
            disconnecting: false,
        }
    }

    fn total_focusable(&self) -> usize {
        self.configs.len() + if self.wifi_connected { 3 } else { 2 }
    }
}

impl Screen for WifiConfigScreen {
    fn on_mount(
        &mut self,
        shared: &SharedState,
        _state_tx: &std::sync::mpsc::Sender<Snapshot>,
    ) -> Command {
        self.loading = true;
        self.wifi_connected = shared.wifi_connected;
        self.disconnecting = false;
        Command::None
    }

    fn handle_msg(&mut self, msg: Msg, _shared: &SharedState) -> Command {
        match msg {
            Msg::Core0Result(result) => {
                match result {
                    Core0Result::WifiListLoaded(list) => {
                        self.configs = list.configs;
                        self.loading = false;
                        self.focused_idx = 0;
                    }
                    Core0Result::WifiListSaved => {
                        self.needs_save = false;
                    }
                    Core0Result::WifiStopped => {
                        self.wifi_connected = false;
                        self.disconnecting = false;
                    }
                    Core0Result::Error(msg) => {
                        log::error!("WifiConfig: error: {}", msg);
                        self.configs = Vec::new();
                        self.loading = false;
                        self.focused_idx = 0;
                        self.disconnecting = false;
                    }
                    _ => {}
                }
                Command::None
            }
            Msg::Key(key_msg) => {
                if self.loading {
                    return Command::None;
                }
                let total = self.total_focusable();
                match key_msg.pressed {
                    Some((KeyEvent::Pressed, PressedSymbol::Esc)) => {
                        Command::SwitchTo(Box::new(MainMenuScreen::default()))
                    }
                    Some((KeyEvent::Pressed, PressedSymbol::ArrowDown)) => {
                        if total > 0 {
                            self.focused_idx = (self.focused_idx + 1) % total;
                        }
                        Command::None
                    }
                    Some((KeyEvent::Pressed, PressedSymbol::ArrowUp)) => {
                        if total > 0 {
                            self.focused_idx = (self.focused_idx + total - 1) % total;
                        }
                        Command::None
                    }
                    Some((KeyEvent::Pressed, PressedSymbol::Enter)) => {
                        let add_idx = self.configs.len();
                        let mut next = add_idx + 1;
                        let disconnect_idx = if self.wifi_connected { let d = next; next += 1; Some(d) } else { None };
                        let back_idx = next;

                        if self.focused_idx == add_idx {
                            Command::SwitchTo(Box::new(WifiConnectScreen::new(self.configs.clone())))
                        } else if disconnect_idx == Some(self.focused_idx) {
                            self.disconnecting = true;
                            Command::None
                        } else if self.focused_idx == back_idx {
                            Command::SwitchTo(Box::new(MainMenuScreen::default()))
                        } else if self.focused_idx < self.configs.len() {
                            let config = self.configs[self.focused_idx].clone();
                            Command::SwitchTo(Box::new(WifiConnectScreen::connect_direct(config, self.configs.clone())))
                        } else {
                            Command::None
                        }
                    }
                    Some((KeyEvent::Pressed, PressedSymbol::Del)) => {
                        // Del key removes the focused config
                        if self.focused_idx < self.configs.len() {
                            self.configs.remove(self.focused_idx);
                            if self.focused_idx >= self.total_focusable() && self.focused_idx > 0 {
                                self.focused_idx -= 1;
                            }
                            self.needs_save = true;
                        }
                        Command::None
                    }
                    _ => Command::None,
                }
            }
            _ => Command::None,
        }
    }

    fn snapshot(&self, shared: &SharedState) -> Snapshot {
        let pending_action = if self.loading {
            Some(Core0Action::LoadWifiList)
        } else if self.needs_save {
            Some(Core0Action::SaveWifiList(WifiConfigList {
                configs: self.configs.clone(),
            }))
        } else if self.disconnecting {
            Some(Core0Action::StopWifi)
        } else {
            None
        };

        Snapshot::WifiConfig(WifiConfigSnapshot {
            configs: self.configs.clone(),
            focused_idx: self.focused_idx,
            loading: self.loading,
            wifi_connected: self.wifi_connected,
            pending_action,
            lang: shared.lang,
        })
    }
}

// ---- Snapshot ----

pub struct WifiConfigSnapshot {
    pub configs: Vec<WifiConfig>,
    pub focused_idx: usize,
    pub loading: bool,
    pub wifi_connected: bool,
    pub pending_action: Option<Core0Action>,
    pub lang: InputLanguage,
}

fn t(en: &'static str, ru: &'static str, lang: InputLanguage) -> &'static str {
    match lang { InputLanguage::En => en, InputLanguage::Ru => ru }
}

impl WifiConfigSnapshot {
    pub fn draw(&self, ui: &mut CardworderUi) {
        const SCREEN_HEIGHT: u32 = 135;
        let viewport_height = SCREEN_HEIGHT - TOP_BAR_HEIGHT;

        let l = self.lang;
        if self.loading {
            ui.draw_text_oneline(
                t("Loading WiFi configs...", "Загрузка конфигов WiFi...", l), CardFont::Medium, ThemeColor::Text,
                embedded_graphics::prelude::Point::new(4, TOP_BAR_HEIGHT as i32 + 10),
                VerticalPosition::Top,
            );
            return;
        }

        // Pre-format SSID labels
        let mut ssid_labels: Vec<heapless::String<48>> = Vec::new();
        for config in &self.configs {
            let mut s = heapless::String::<48>::new();
            let _ = write!(s, "> {}", config.ssid.as_str());
            ssid_labels.push(s);
        }

        let mut lines: Vec<UiLineType> = Vec::new();

        // Config entries
        for (idx, label) in ssid_labels.iter().enumerate() {
            let color = if idx == self.focused_idx { ThemeColor::Selected } else { ThemeColor::Text };
            lines.push(UiLineType::Elements(vec![
                UiLineElement::Text(label.as_str(), CardFont::Medium, VerticalPosition::Top, color),
            ]));
        }

        if self.configs.is_empty() {
            lines.push(UiLineType::Elements(vec![
                UiLineElement::Text(t("(none saved)", "(нет сохранённых)", l), CardFont::Medium, VerticalPosition::Top, ThemeColor::Text),
            ]));
        }

        lines.push(UiLineType::Spacer(4));

        // "Add New" button
        let add_idx = self.configs.len();
        let mut next_idx = add_idx + 1;
        let add_color = if self.focused_idx == add_idx { ThemeColor::Selected } else { ThemeColor::Text };
        lines.push(UiLineType::Elements(vec![
            UiLineElement::Text(t("+ Scan & Add WiFi", "+ Поиск и добавление", l), CardFont::Medium, VerticalPosition::Top, add_color),
        ]));

        // "Disconnect" button (only if connected)
        if self.wifi_connected {
            let disc_color = if self.focused_idx == next_idx { ThemeColor::Selected } else { ThemeColor::Error };
            lines.push(UiLineType::Elements(vec![
                UiLineElement::Text(t("[Disconnect WiFi]", "[Отключить WiFi]", l), CardFont::Medium, VerticalPosition::Top, disc_color),
            ]));
            next_idx += 1;
        }

        // "Back" button
        let back_color = if self.focused_idx == next_idx { ThemeColor::Selected } else { ThemeColor::Text };
        lines.push(UiLineType::Elements(vec![
            UiLineElement::Text(t("<- Back", "<- Назад", l), CardFont::Medium, VerticalPosition::Top, back_color),
        ]));

        // Hint
        if self.focused_idx < self.configs.len() {
            lines.push(UiLineType::Spacer(4));
            lines.push(UiLineType::Elements(vec![
                UiLineElement::Text(t("Fn+Bksp=Delete", "Fn+Bksp=Удалить", l), CardFont::Small, VerticalPosition::Top, ThemeColor::Color(embedded_graphics::pixelcolor::Rgb565::CSS_GRAY)),
            ]));
        }

        let scroll_target = if self.configs.is_empty() {
            match self.focused_idx {
                0 => 2, // add (after empty + spacer)
                _ => 3, // back
            }
        } else if self.focused_idx < self.configs.len() {
            self.focused_idx
        } else {
            self.configs.len() + 1 + (self.focused_idx - self.configs.len())
        };

        let composed = compose_scrolled_form(&lines, scroll_target.min(lines.len().saturating_sub(1)), viewport_height, TOP_BAR_HEIGHT as i32, ui);
        render_visible_lines(&composed, &lines, ui);
    }
}
