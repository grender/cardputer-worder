use core::fmt::Write;
use u8g2_fonts::types::VerticalPosition;

use crate::input::keyboard::{InputLanguage, PressedSymbol};
use crate::input::keyboard_io::KeyEvent;
use crate::screen::{Screen, Snapshot};
use crate::screens::wifi_config::WifiConfigScreen;
use crate::types::{
    Command, Core0Action, Core0Result, Msg, ScannedNetwork, SharedState, WifiConfig, WifiConfigList,
};
use crate::ui::cardworder_ui::{CardFont, CardworderUi, ThemeColor, TOP_BAR_HEIGHT};
use crate::ui::elements::{UiLineElement, UiLineType};
use crate::ui::framebuffer::CardworderFB;

#[derive(Clone)]
enum Phase { StartingWifi, Scanning, ShowingResults, EnteringPassword, Connecting, Connected, Saving, Error }

fn ts(en: &str, ru: &str, lang: InputLanguage) -> String {
    match lang { InputLanguage::En => en, InputLanguage::Ru => ru }.to_string()
}
fn t(en: &'static str, ru: &'static str, lang: InputLanguage) -> &'static str {
    match lang { InputLanguage::En => en, InputLanguage::Ru => ru }
}

pub struct WifiConnectScreen {
    phase: Phase,
    saved_configs: Vec<WifiConfig>,
    networks: Vec<ScannedNetwork>,
    selected_network_idx: usize,
    password: heapless::String<64>,
    password_cursor: usize,
    focused_idx: usize,
    status_text: String,
    direct_connect: Option<WifiConfig>,
}

impl WifiConnectScreen {
    pub fn new(saved_configs: Vec<WifiConfig>) -> Self {
        Self {
            phase: Phase::StartingWifi, saved_configs, networks: Vec::new(),
            selected_network_idx: 0, password: heapless::String::new(), password_cursor: 0,
            focused_idx: 0, status_text: "Starting WiFi...".to_string(), direct_connect: None,
        }
    }

    pub fn connect_direct(config: WifiConfig, saved_configs: Vec<WifiConfig>) -> Self {
        Self {
            phase: Phase::StartingWifi, saved_configs, networks: Vec::new(),
            selected_network_idx: 0, password: config.password.clone(), password_cursor: 0,
            focused_idx: 0, status_text: "Starting WiFi...".to_string(), direct_connect: Some(config),
        }
    }

    fn find_saved_password(&self, ssid: &str) -> Option<&heapless::String<64>> {
        self.saved_configs.iter().find(|c| c.ssid.as_str() == ssid).map(|c| &c.password)
    }

    fn total_focusable(&self) -> usize {
        match self.phase {
            Phase::ShowingResults => self.networks.len() + 1,
            Phase::EnteringPassword => 3,
            _ => 1,
        }
    }

    fn handle_input_key(&mut self, symbol: PressedSymbol) {
        match symbol {
            PressedSymbol::Char(c) => {
                if self.password.len() < 64 {
                    let pos = byte_pos(self.password.as_str(), self.password_cursor);
                    let mut nv = heapless::String::<64>::new();
                    let _ = nv.push_str(&self.password.as_str()[..pos]);
                    let _ = nv.push(c);
                    let _ = nv.push_str(&self.password.as_str()[pos..]);
                    self.password = nv;
                    self.password_cursor += 1;
                }
            }
            PressedSymbol::Backspace => {
                if self.password_cursor > 0 {
                    let p = byte_pos(self.password.as_str(), self.password_cursor - 1);
                    let c = byte_pos(self.password.as_str(), self.password_cursor);
                    let mut nv = heapless::String::<64>::new();
                    let _ = nv.push_str(&self.password.as_str()[..p]);
                    let _ = nv.push_str(&self.password.as_str()[c..]);
                    self.password = nv;
                    self.password_cursor -= 1;
                }
            }
            PressedSymbol::ArrowLeft => { if self.password_cursor > 0 { self.password_cursor -= 1; } }
            PressedSymbol::ArrowRight => { if self.password_cursor < self.password.chars().count() { self.password_cursor += 1; } }
            _ => {}
        }
    }
}

fn byte_pos(s: &str, n: usize) -> usize {
    s.char_indices().nth(n).map(|(i, _)| i).unwrap_or(s.len())
}

impl Screen for WifiConnectScreen {
    fn on_mount(&mut self, shared: &SharedState, _state_tx: &std::sync::mpsc::Sender<Snapshot>) -> Command {
        self.phase = Phase::StartingWifi;
        self.status_text = ts("Starting WiFi...", "Запуск WiFi...", shared.lang);
        Command::None
    }

    fn handle_msg(&mut self, msg: Msg, shared: &SharedState) -> Command {
        let l = shared.lang;
        match msg {
            Msg::Core0Result(result) => {
                match result {
                    Core0Result::WifiStarted => {
                        if self.direct_connect.is_some() {
                            self.phase = Phase::Connecting;
                            self.status_text = ts("Connecting...", "Подключение...", l);
                        } else {
                            self.phase = Phase::Scanning;
                            self.status_text = ts("Scanning...", "Поиск сетей...", l);
                        }
                    }
                    Core0Result::WifiScanResults(mut networks) => {
                        for net in networks.iter_mut() {
                            net.has_saved_password = self.find_saved_password(net.ssid.as_str()).is_some();
                        }
                        networks.sort_by(|a, b| b.has_saved_password.cmp(&a.has_saved_password).then(b.signal_strength.cmp(&a.signal_strength)));
                        self.networks = networks;
                        self.phase = Phase::ShowingResults;
                        self.focused_idx = 0;
                    }
                    Core0Result::WifiConnected { .. } => {
                        if self.selected_network_idx < self.networks.len() {
                            let ssid = self.networks[self.selected_network_idx].ssid.clone();
                            if self.find_saved_password(ssid.as_str()).is_none() {
                                self.saved_configs.push(WifiConfig { ssid, password: self.password.clone() });
                            }
                        }
                        self.phase = Phase::Saving;
                        self.status_text = ts("Saving config...", "Сохранение...", l);
                    }
                    Core0Result::WifiListSaved => {
                        self.phase = Phase::Connected;
                        self.focused_idx = 0;
                        self.status_text = ts("Connected!", "Подключено!", l);
                    }
                    Core0Result::WifiStopped => {
                        return Command::SwitchTo(Box::new(WifiConfigScreen::new()));
                    }
                    Core0Result::Error(msg) => {
                        log::error!("WifiConnect: error: {}", msg);
                        self.status_text = msg;
                        self.phase = Phase::Error;
                    }
                    _ => {}
                }
                Command::None
            }
            Msg::Key(key_msg) => {
                let total = self.total_focusable();
                match key_msg.pressed {
                    Some((KeyEvent::Pressed, PressedSymbol::Esc)) => {
                        return Command::SwitchTo(Box::new(WifiConfigScreen::new()));
                    }
                    Some((KeyEvent::Pressed, PressedSymbol::ArrowDown)) => {
                        if total > 0 { self.focused_idx = (self.focused_idx + 1) % total; }
                    }
                    Some((KeyEvent::Pressed, PressedSymbol::ArrowUp)) => {
                        if total > 0 { self.focused_idx = (self.focused_idx + total - 1) % total; }
                    }
                    Some((KeyEvent::Pressed, PressedSymbol::Enter)) => {
                        match self.phase {
                            Phase::ShowingResults => {
                                if self.focused_idx < self.networks.len() {
                                    self.selected_network_idx = self.focused_idx;
                                    let ssid = &self.networks[self.focused_idx].ssid;
                                    if let Some(pwd) = self.find_saved_password(ssid.as_str()) {
                                        self.password = pwd.clone();
                                        self.phase = Phase::Connecting;
                                        self.status_text = ts("Connecting...", "Подключение...", l);
                                    } else {
                                        self.password = heapless::String::new();
                                        self.password_cursor = 0;
                                        self.phase = Phase::EnteringPassword;
                                        self.focused_idx = 0;
                                    }
                                } else {
                                    return Command::SwitchTo(Box::new(WifiConfigScreen::new()));
                                }
                            }
                            Phase::EnteringPassword => {
                                if self.focused_idx == 1 {
                                    self.phase = Phase::Connecting;
                                    self.status_text = ts("Connecting...", "Подключение...", l);
                                } else if self.focused_idx == 2 {
                                    self.phase = Phase::ShowingResults;
                                    self.focused_idx = 0;
                                }
                            }
                            Phase::Connected => {
                                return Command::SwitchTo(Box::new(WifiConfigScreen::new()));
                            }
                            Phase::Error => {
                                return Command::SwitchTo(Box::new(WifiConfigScreen::new()));
                            }
                            _ => {}
                        }
                    }
                    Some((KeyEvent::Pressed, symbol)) => {
                        if matches!(self.phase, Phase::EnteringPassword) && self.focused_idx == 0 {
                            self.handle_input_key(symbol);
                        }
                    }
                    _ => {}
                }
                Command::None
            }
            _ => Command::None,
        }
    }

    fn snapshot(&self, shared: &SharedState) -> Snapshot {
        let pending_action = match self.phase {
            Phase::StartingWifi => Some(Core0Action::StartWifi),
            Phase::Scanning => Some(Core0Action::StartWifiScan),
            Phase::Connecting => {
                if let Some(ref config) = self.direct_connect {
                    Some(Core0Action::ConnectWifi(config.clone()))
                } else if self.selected_network_idx < self.networks.len() {
                    let ssid = self.networks[self.selected_network_idx].ssid.clone();
                    Some(Core0Action::ConnectWifi(WifiConfig { ssid, password: self.password.clone() }))
                } else { None }
            }
            Phase::Saving => Some(Core0Action::SaveWifiList(WifiConfigList { configs: self.saved_configs.clone() })),
            _ => None,
        };
        Snapshot::WifiConnect(WifiConnectSnapshot {
            phase: self.phase.clone(), networks: self.networks.clone(),
            focused_idx: self.focused_idx, password: self.password.clone(),
            password_cursor: self.password_cursor, status_text: self.status_text.clone(),
            pending_action, lang: shared.lang,
        })
    }
}

// ---- Snapshot ----

pub struct WifiConnectSnapshot {
    pub phase: Phase, pub networks: Vec<ScannedNetwork>, pub focused_idx: usize,
    pub password: heapless::String<64>, pub password_cursor: usize, pub status_text: String,
    pub pending_action: Option<Core0Action>, pub lang: InputLanguage,
}

impl WifiConnectSnapshot {
    pub fn draw<FB: CardworderFB>(&self, ui: &mut CardworderUi<FB>) {
        use embedded_graphics::prelude::Point;
        use crate::ui::render::{compose_scrolled_form, render_visible_lines};
        let font = CardFont::Medium;
        let small = CardFont::Small;
        let line_h = ui.font_height(font) as i32 + 3;
        let mut y = TOP_BAR_HEIGHT as i32 + 2;
        let l = self.lang;

        match self.phase {
            Phase::StartingWifi => {
                ui.draw_text_oneline(self.status_text.as_str(), font, ThemeColor::Text, Point::new(4, y), VerticalPosition::Top);
            }
            Phase::Scanning => {
                ui.draw_text_oneline(self.status_text.as_str(), font, ThemeColor::Text, Point::new(4, y), VerticalPosition::Top);
            }
            Phase::ShowingResults => {
                const SCREEN_HEIGHT: u32 = 135;
                let viewport_height = SCREEN_HEIGHT - TOP_BAR_HEIGHT;
                let mut net_labels: Vec<heapless::String<48>> = Vec::new();
                for net in &self.networks {
                    let mut s = heapless::String::<48>::new();
                    let saved = if net.has_saved_password { " *" } else { "" };
                    let _ = write!(s, "{} ({}dB){}", net.ssid.as_str(), net.signal_strength, saved);
                    net_labels.push(s);
                }
                let mut lines: Vec<UiLineType> = Vec::new();
                for (idx, label) in net_labels.iter().enumerate() {
                    let color = if idx == self.focused_idx { ThemeColor::Selected } else { ThemeColor::Text };
                    lines.push(UiLineType::Elements(vec![UiLineElement::Text(label.as_str(), font, VerticalPosition::Top, color)]));
                }
                if self.networks.is_empty() {
                    lines.push(UiLineType::Elements(vec![UiLineElement::Text(t("No networks found", "Сети не найдены", l), font, VerticalPosition::Top, ThemeColor::Error)]));
                }
                lines.push(UiLineType::Spacer(2));
                let back_color = if self.focused_idx == self.networks.len() { ThemeColor::Selected } else { ThemeColor::Text };
                lines.push(UiLineType::Elements(vec![UiLineElement::Text(t("<- Back", "<- Назад", l), font, VerticalPosition::Top, back_color)]));
                let scroll_target = if self.focused_idx < self.networks.len() { self.focused_idx } else { lines.len().saturating_sub(1) };
                let composed = compose_scrolled_form(&lines, scroll_target, viewport_height, TOP_BAR_HEIGHT as i32, ui);
                render_visible_lines(&composed, &lines, ui);
            }
            Phase::EnteringPassword => {
                ui.draw_text_oneline(t("Enter Password", "Введите пароль", l), small, ThemeColor::Text, Point::new(4, y), VerticalPosition::Top);
                y += ui.font_height(small) as i32 + 2;
                let lines = vec![UiLineType::InputField { label: t("Password", "Пароль", l), value: self.password.clone(), cursor_pos: self.password_cursor, focused: self.focused_idx == 0 }];
                let composed = compose_scrolled_form(&lines, 0, 40, y, ui);
                render_visible_lines(&composed, &lines, ui);
                y += 30;
                let connect_color = if self.focused_idx == 1 { ThemeColor::Selected } else { ThemeColor::Text };
                ui.draw_text_oneline(t("[Connect]", "[Подключить]", l), font, connect_color, Point::new(4, y), VerticalPosition::Top);
                y += line_h;
                let back_color = if self.focused_idx == 2 { ThemeColor::Selected } else { ThemeColor::Text };
                ui.draw_text_oneline(t("<- Back", "<- Назад", l), font, back_color, Point::new(4, y), VerticalPosition::Top);
            }
            Phase::Connecting => {
                ui.draw_text_oneline(self.status_text.as_str(), font, ThemeColor::Text, Point::new(4, y), VerticalPosition::Top);
            }
            Phase::Saving => {
                ui.draw_text_oneline(self.status_text.as_str(), font, ThemeColor::Text, Point::new(4, y), VerticalPosition::Top);
            }
            Phase::Connected => {
                ui.draw_text_oneline(t("Connected!", "Подключено!", l), font, ThemeColor::Color(embedded_graphics::pixelcolor::Rgb565::new(0, 50, 0)), Point::new(4, y), VerticalPosition::Top);
                y += line_h + 4;
                ui.draw_text_oneline(t("<- Back", "<- Назад", l), font, ThemeColor::Selected, Point::new(4, y), VerticalPosition::Top);
            }
            Phase::Error => {
                ui.draw_text_oneline(t("Error:", "Ошибка:", l), font, ThemeColor::Error, Point::new(4, y), VerticalPosition::Top);
                y += line_h;
                ui.draw_text_oneline(self.status_text.as_str(), small, ThemeColor::Text, Point::new(4, y), VerticalPosition::Top);
                y += ui.font_height(small) as i32 + 4;
                ui.draw_text_oneline(t("Press any key", "Нажмите клавишу", l), small, ThemeColor::Selected, Point::new(4, y), VerticalPosition::Top);
            }
        }
    }
}
