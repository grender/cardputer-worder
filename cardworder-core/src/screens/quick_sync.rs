use core::fmt::Write;
use embedded_graphics::prelude::Point;
use u8g2_fonts::types::VerticalPosition;

use crate::input::keyboard::{InputLanguage, PressedSymbol};
use crate::input::keyboard_io::KeyEvent;
use crate::screen::{Screen, Snapshot};
use crate::screens::main_menu::MainMenuScreen;
use crate::types::{Command, Core0Action, Core0Result, Msg, ScannedNetwork, SharedState, WifiConfig};
use crate::ui::cardworder_ui::{CardFont, CardworderUi, ThemeColor, TOP_BAR_HEIGHT};
use crate::ui::framebuffer::CardworderFB;

fn t(en: &'static str, ru: &'static str, lang: InputLanguage) -> &'static str {
    match lang { InputLanguage::En => en, InputLanguage::Ru => ru }
}

fn now_us() -> u64 {
    use std::sync::OnceLock;
    use std::time::Instant;
    static START: OnceLock<Instant> = OnceLock::new();
    START.get_or_init(Instant::now).elapsed().as_micros() as u64
}

#[derive(Clone)]
enum Phase {
    LoadingConfigs,
    StartingWifi,
    Scanning,
    Connecting,
    SettingTimezone,
    SyncingNtp,
    WaitingNtp,
    Disconnecting,
    Done,
    Error,
    NoConfigs,
}

impl Phase {
    fn timeout_us(&self) -> u64 {
        match self {
            Phase::LoadingConfigs => 5_000_000,
            Phase::StartingWifi => 10_000_000,
            Phase::Scanning => 15_000_000,
            Phase::Connecting => 15_000_000,
            Phase::SettingTimezone => 3_000_000,
            Phase::SyncingNtp => 3_000_000,
            Phase::WaitingNtp => 15_000_000,
            Phase::Disconnecting => 5_000_000,
            _ => u64::MAX,
        }
    }

    fn status(&self, lang: InputLanguage) -> &'static str {
        match self {
            Phase::LoadingConfigs => t("Loading WiFi configs...", "Загрузка WiFi конфигов...", lang),
            Phase::StartingWifi => t("Starting WiFi...", "Запуск WiFi...", lang),
            Phase::Scanning => t("Scanning networks...", "Поиск сетей...", lang),
            Phase::Connecting => t("Connecting...", "Подключение...", lang),
            Phase::SettingTimezone => t("Setting timezone...", "Установка часового пояса...", lang),
            Phase::SyncingNtp => t("Starting NTP sync...", "Запуск синхронизации NTP...", lang),
            Phase::WaitingNtp => t("Waiting for NTP sync...", "Ожидание синхронизации NTP...", lang),
            Phase::Disconnecting => t("Time synced! Disconnecting...", "Время синхронизировано! Отключение...", lang),
            Phase::Done => t("Done!", "Готово!", lang),
            Phase::NoConfigs => t("No saved WiFi configs", "Нет сохранённых WiFi", lang),
            Phase::Error => t("Error", "Ошибка", lang),
        }
    }

    fn timeout_msg(&self, lang: InputLanguage) -> &'static str {
        match self {
            Phase::LoadingConfigs => t("Load timeout", "Таймаут загрузки", lang),
            Phase::StartingWifi => t("WiFi start timeout", "Таймаут запуска WiFi", lang),
            Phase::Scanning => t("Scan timeout", "Таймаут сканирования", lang),
            Phase::Connecting => t("All networks failed", "Все сети недоступны", lang),
            Phase::SettingTimezone => t("Timezone timeout", "Таймаут часового пояса", lang),
            Phase::SyncingNtp => t("NTP start timeout", "Таймаут запуска NTP", lang),
            Phase::WaitingNtp => t("NTP timeout", "Таймаут NTP", lang),
            Phase::Disconnecting => t("Disconnect timeout", "Таймаут отключения", lang),
            _ => "",
        }
    }

    fn action(&self, screen: &QuickSyncScreen) -> Option<Core0Action> {
        match self {
            Phase::LoadingConfigs => Some(Core0Action::LoadWifiList),
            Phase::StartingWifi => Some(Core0Action::StartWifi),
            Phase::Scanning => Some(Core0Action::StartWifiScan),
            Phase::Connecting => screen.current_config().map(|c| Core0Action::ConnectWifi(c.clone())),
            Phase::SettingTimezone => Some(Core0Action::SetTimezone),
            Phase::SyncingNtp => Some(Core0Action::StartNtp),
            Phase::WaitingNtp => Some(Core0Action::CheckNtpStatus),
            Phase::Disconnecting => Some(Core0Action::StopWifi),
            _ => None,
        }
    }
}

pub struct QuickSyncScreen {
    phase: Phase,
    saved_configs: Vec<WifiConfig>,
    matched_configs: Vec<WifiConfig>,
    current_config_idx: usize,
    phase_start_us: u64,
    status_override: Option<String>,
}

impl QuickSyncScreen {
    pub fn new() -> Self {
        Self {
            phase: Phase::LoadingConfigs,
            saved_configs: Vec::new(),
            matched_configs: Vec::new(),
            current_config_idx: 0,
            phase_start_us: now_us(),
            status_override: None,
        }
    }

    fn go(&mut self, phase: Phase) {
        self.phase = phase;
        self.status_override = None;
        self.phase_start_us = now_us();
    }

    fn fail(&mut self, msg: &str) {
        self.phase = Phase::Error;
        self.status_override = Some(msg.to_string());
        self.phase_start_us = now_us();
    }

    fn current_config(&self) -> Option<&WifiConfig> {
        self.matched_configs.get(self.current_config_idx)
    }

    fn status_text(&self, lang: InputLanguage) -> String {
        if let Some(ref s) = self.status_override {
            return s.clone();
        }
        self.phase.status(lang).to_string()
    }

    fn check_timeout(&mut self, lang: InputLanguage) -> bool {
        let elapsed = now_us() - self.phase_start_us;
        if elapsed <= self.phase.timeout_us() {
            return false;
        }

        if matches!(self.phase, Phase::Connecting) {
            self.current_config_idx += 1;
            if let Some(cfg) = self.current_config() {
                let mut msg = String::from("Trying ");
                msg.push_str(cfg.ssid.as_str());
                msg.push_str("...");
                self.status_override = Some(msg);
                self.phase_start_us = now_us();
                return true;
            }
        }

        self.fail(self.phase.timeout_msg(lang));
        true
    }

    fn match_networks(&mut self, scan_results: &[ScannedNetwork]) {
        let mut matches: Vec<(WifiConfig, i8)> = Vec::new();
        for net in scan_results {
            if let Some(saved) = self.saved_configs.iter().find(|c| c.ssid == net.ssid) {
                matches.push((saved.clone(), net.signal_strength));
            }
        }
        matches.sort_by(|a, b| b.1.cmp(&a.1));
        self.matched_configs = matches.into_iter().map(|(c, _)| c).collect();
    }
}

impl Default for QuickSyncScreen {
    fn default() -> Self { Self::new() }
}

impl Screen for QuickSyncScreen {
    fn on_mount(&mut self, _shared: &SharedState, _state_tx: &std::sync::mpsc::Sender<Snapshot>) -> Command {
        self.go(Phase::LoadingConfigs);
        Command::None
    }

    fn handle_msg(&mut self, msg: Msg, shared: &SharedState) -> Command {
        let l = shared.lang;
        if self.check_timeout(l) {
            return Command::None;
        }

        match msg {
            Msg::Core0Result(result) => self.handle_result(result, l),
            Msg::Key(key_msg) => {
                if let Some((KeyEvent::Pressed, PressedSymbol::Esc)) = key_msg.pressed {
                    return Command::SwitchTo(Box::new(MainMenuScreen::default()));
                }
                if let Some((KeyEvent::Pressed, PressedSymbol::Enter)) = key_msg.pressed {
                    if matches!(self.phase, Phase::Error | Phase::NoConfigs) {
                        return Command::SwitchTo(Box::new(MainMenuScreen::default()));
                    }
                }
                Command::None
            }
            _ => Command::None,
        }
    }

    fn snapshot(&self, shared: &SharedState) -> Snapshot {
        Snapshot::QuickSync(QuickSyncSnapshot {
            phase: self.phase.clone(),
            matched_configs_len: self.matched_configs.len(),
            current_config_idx: self.current_config_idx,
            status_text: self.status_text(shared.lang),
            pending_action: self.phase.action(self),
            lang: shared.lang,
        })
    }
}

impl QuickSyncScreen {
    fn handle_result(&mut self, result: Core0Result, lang: InputLanguage) -> Command {
        match result {
            Core0Result::WifiListLoaded(list) => {
                self.saved_configs = list.configs;
                if self.saved_configs.is_empty() {
                    self.go(Phase::NoConfigs);
                } else {
                    self.go(Phase::StartingWifi);
                }
            }
            Core0Result::WifiStarted => self.go(Phase::Scanning),
            Core0Result::WifiScanResults(networks) => {
                self.match_networks(&networks);
                if self.matched_configs.is_empty() {
                    self.fail(t("No known networks nearby", "Нет известных сетей поблизости", lang));
                } else {
                    self.current_config_idx = 0;
                    let ssid = self.matched_configs[0].ssid.clone();
                    let mut msg = String::from("Connecting to ");
                    msg.push_str(ssid.as_str());
                    msg.push_str("...");
                    self.go(Phase::Connecting);
                    self.status_override = Some(msg);
                }
            }
            Core0Result::WifiConnected { .. } => self.go(Phase::SettingTimezone),
            Core0Result::TimezoneSet => self.go(Phase::SyncingNtp),
            Core0Result::NtpStarted => self.go(Phase::WaitingNtp),
            Core0Result::NtpSynced(done) => {
                if done { self.go(Phase::Disconnecting); }
            }
            Core0Result::WifiStopped => {
                return Command::SwitchTo(Box::new(MainMenuScreen::default()));
            }
            Core0Result::Error(msg) => {
                log::error!("QuickSync: error: {}", msg);
                self.fail(&msg);
            }
            _ => {}
        }
        Command::None
    }
}

pub struct QuickSyncSnapshot {
    pub phase: Phase,
    pub matched_configs_len: usize,
    pub current_config_idx: usize,
    pub status_text: String,
    pub pending_action: Option<Core0Action>,
    pub lang: InputLanguage,
}

impl QuickSyncSnapshot {
    pub fn draw<FB: CardworderFB>(&self, ui: &mut CardworderUi<FB>) {
        let l = self.lang;
        let mut y = TOP_BAR_HEIGHT as i32 + 10;

        ui.draw_text_oneline(
            t("Quick Sync", "Быстрая синхронизация", l),
            CardFont::Large, ThemeColor::Selected,
            Point::new(4, y), VerticalPosition::Top,
        );
        y += ui.font_height(CardFont::Large) as i32 + 8;

        ui.draw_text_oneline(
            self.status_text.as_str(),
            CardFont::Medium, ThemeColor::Text,
            Point::new(4, y), VerticalPosition::Top,
        );
        y += ui.font_height(CardFont::Medium) as i32 + 8;

        if matches!(self.phase, Phase::Connecting) && self.matched_configs_len > 1 {
            let mut prog = heapless::String::<32>::new();
            let _ = write!(prog, "({}/{})", self.current_config_idx + 1, self.matched_configs_len);
            ui.draw_text_oneline(
                prog.as_str(),
                CardFont::Small, ThemeColor::Text,
                Point::new(4, y), VerticalPosition::Top,
            );
            y += ui.font_height(CardFont::Small) as i32 + 4;
        }

        let hint = if matches!(self.phase, Phase::Error | Phase::NoConfigs) {
            t("Press Enter/Esc", "Нажмите Enter/Esc", l)
        } else {
            t("Press Esc to cancel", "Esc для отмены", l)
        };
        ui.draw_text_oneline(hint, CardFont::Small, ThemeColor::Text, Point::new(4, y), VerticalPosition::Top);
    }
}
