use core::fmt::Write;
use embedded_graphics::prelude::Point;
use u8g2_fonts::types::VerticalPosition;

use crate::cardputer_hal::input::keyboard::{InputLanguage, PressedSymbol};
use crate::cardputer_hal::input::keyboard_io::KeyEvent;
use crate::cardputer_hal::wifi::wifi::WifiConfig;
use crate::screen::{Screen, Snapshot};
use crate::screens::main_menu::MainMenuScreen;
use crate::types::{Command, Core0Action, Core0Result, Msg, ScannedNetwork, SharedState};
use crate::ui::cardworder_ui::{CardFont, CardworderUi, ThemeColor, TOP_BAR_HEIGHT};

const TIMEOUT_LOAD: u64 = 5_000_000;
const TIMEOUT_WIFI_START: u64 = 10_000_000;
const TIMEOUT_SCAN: u64 = 15_000_000;
const TIMEOUT_CONNECT: u64 = 15_000_000;
const TIMEOUT_TZ: u64 = 3_000_000;
const TIMEOUT_NTP_START: u64 = 3_000_000;
const TIMEOUT_NTP_WAIT: u64 = 15_000_000;
const TIMEOUT_DISCONNECT: u64 = 5_000_000;

fn t(en: &'static str, ru: &'static str, lang: InputLanguage) -> &'static str {
    match lang {
        InputLanguage::En => en,
        InputLanguage::Ru => ru,
    }
}

fn now_us() -> u64 {
    unsafe { esp_idf_svc::sys::esp_timer_get_time() as u64 }
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

pub struct QuickSyncScreen {
    phase: Phase,
    saved_configs: Vec<WifiConfig>,
    matched_configs: Vec<WifiConfig>,
    current_config_idx: usize,
    phase_start_us: u64,
    status_text: String,
}

impl QuickSyncScreen {
    pub fn new() -> Self {
        Self {
            phase: Phase::LoadingConfigs,
            saved_configs: Vec::new(),
            matched_configs: Vec::new(),
            current_config_idx: 0,
            phase_start_us: now_us(),
            status_text: String::new(),
        }
    }

    fn set_phase(&mut self, phase: Phase, status: &str) {
        self.phase = phase;
        self.status_text = status.to_string();
        self.phase_start_us = now_us();
    }

    fn elapsed_us(&self) -> u64 {
        now_us() - self.phase_start_us
    }

    fn check_timeout(&mut self, lang: InputLanguage) -> bool {
        match self.phase {
            Phase::LoadingConfigs if self.elapsed_us() > TIMEOUT_LOAD => {
                self.set_phase(Phase::Error, t("Load timeout", "Таймаут загрузки", lang));
                true
            }
            Phase::StartingWifi if self.elapsed_us() > TIMEOUT_WIFI_START => {
                self.set_phase(Phase::Error, t("WiFi start timeout", "Таймаут запуска WiFi", lang));
                true
            }
            Phase::Scanning if self.elapsed_us() > TIMEOUT_SCAN => {
                self.set_phase(Phase::Error, t("Scan timeout", "Таймаут сканирования", lang));
                true
            }
            Phase::Connecting if self.elapsed_us() > TIMEOUT_CONNECT => {
                self.current_config_idx += 1;
                if self.current_config_idx < self.matched_configs.len() {
                    let ssid = self.matched_configs[self.current_config_idx].ssid.clone();
                    let mut msg = String::from("Trying ");
                    msg.push_str(ssid.as_str());
                    msg.push_str("...");
                    self.set_phase(Phase::Connecting, &msg);
                } else {
                    self.set_phase(Phase::Error, t("All networks failed", "Все сети недоступны", lang));
                }
                true
            }
            Phase::SettingTimezone if self.elapsed_us() > TIMEOUT_TZ => {
                self.set_phase(Phase::Error, t("Timezone timeout", "Таймаут часового пояса", lang));
                true
            }
            Phase::SyncingNtp if self.elapsed_us() > TIMEOUT_NTP_START => {
                self.set_phase(Phase::Error, t("NTP start timeout", "Таймаут запуска NTP", lang));
                true
            }
            Phase::WaitingNtp if self.elapsed_us() > TIMEOUT_NTP_WAIT => {
                self.set_phase(Phase::Error, t("NTP timeout", "Таймаут NTP", lang));
                true
            }
            Phase::Disconnecting if self.elapsed_us() > TIMEOUT_DISCONNECT => {
                self.set_phase(Phase::Error, t("Disconnect timeout", "Таймаут отключения", lang));
                true
            }
            _ => false,
        }
    }

    fn match_networks(&mut self, scan_results: &[ScannedNetwork]) {
        self.matched_configs.clear();
        let mut matches: Vec<(WifiConfig, i8)> = Vec::new();
        for net in scan_results {
            if let Some(saved) = self.saved_configs.iter().find(|c| c.ssid == net.ssid) {
                matches.push((saved.clone(), net.signal_strength));
            }
        }
        // Sort by signal strength (strongest first)
        matches.sort_by(|a, b| b.1.cmp(&a.1));
        self.matched_configs = matches.into_iter().map(|(c, _)| c).collect();
    }
}

impl Default for QuickSyncScreen {
    fn default() -> Self {
        Self::new()
    }
}

impl Screen for QuickSyncScreen {
    fn on_mount(
        &mut self,
        _shared: &SharedState,
        _state_tx: &std::sync::mpsc::Sender<Snapshot>,
    ) -> Command {
        self.set_phase(Phase::LoadingConfigs, "Loading WiFi configs...");
        Command::None
    }

    fn handle_msg(&mut self, msg: Msg, shared: &SharedState) -> Command {
        let l = shared.lang;

        // Check timeout on every message
        if self.check_timeout(l) {
            return Command::None;
        }

        match msg {
            Msg::Core0Result(result) => {
                match result {
                    Core0Result::WifiListLoaded(list) => {
                        self.saved_configs = list.configs;
                        if self.saved_configs.is_empty() {
                            self.set_phase(
                                Phase::NoConfigs,
                                t("No saved WiFi configs", "Нет сохранённых WiFi", l),
                            );
                        } else {
                            self.set_phase(
                                Phase::StartingWifi,
                                t("Starting WiFi...", "Запуск WiFi...", l),
                            );
                        }
                    }
                    Core0Result::WifiStarted => {
                        self.set_phase(
                            Phase::Scanning,
                            t("Scanning networks...", "Поиск сетей...", l),
                        );
                    }
                    Core0Result::WifiScanResults(networks) => {
                        self.match_networks(&networks);
                        if self.matched_configs.is_empty() {
                            self.set_phase(
                                Phase::Error,
                                t(
                                    "No known networks nearby",
                                    "Нет известных сетей поблизости",
                                    l,
                                ),
                            );
                        } else {
                            self.current_config_idx = 0;
                            let ssid = self.matched_configs[0].ssid.clone();
                            let mut msg = String::from("Connecting to ");
                            msg.push_str(ssid.as_str());
                            msg.push_str("...");
                            self.set_phase(Phase::Connecting, &msg);
                        }
                    }
                    Core0Result::WifiConnected { .. } => {
                        self.set_phase(
                            Phase::SettingTimezone,
                            t("Setting timezone...", "Установка часового пояса...", l),
                        );
                    }
                    Core0Result::TimezoneSet => {
                        self.set_phase(
                            Phase::SyncingNtp,
                            t("Starting NTP sync...", "Запуск синхронизации NTP...", l),
                        );
                    }
                    Core0Result::NtpStarted => {
                        self.set_phase(
                            Phase::WaitingNtp,
                            t("Waiting for NTP sync...", "Ожидание синхронизации NTP...", l),
                        );
                    }
                    Core0Result::NtpSynced(done) => {
                        if done {
                            self.set_phase(
                                Phase::Disconnecting,
                                t(
                                    "Time synced! Disconnecting...",
                                    "Время синхронизировано! Отключение...",
                                    l,
                                ),
                            );
                        }
                        // If not done, keep waiting (timeout will catch it)
                    }
                    Core0Result::WifiStopped => {
                        return Command::SwitchTo(Box::new(MainMenuScreen::default()));
                    }
                    Core0Result::Error(msg) => {
                        log::error!("QuickSync: error: {}", msg);
                        self.set_phase(Phase::Error, &msg);
                    }
                    _ => {}
                }
                Command::None
            }
            Msg::Key(key_msg) => {
                match key_msg.pressed {
                    Some((KeyEvent::Pressed, PressedSymbol::Esc)) => {
                        return Command::SwitchTo(Box::new(MainMenuScreen::default()));
                    }
                    Some((KeyEvent::Pressed, PressedSymbol::Enter)) => {
                        if matches!(self.phase, Phase::Error | Phase::NoConfigs) {
                            return Command::SwitchTo(Box::new(MainMenuScreen::default()));
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
            Phase::LoadingConfigs => Some(Core0Action::LoadWifiList),
            Phase::StartingWifi => Some(Core0Action::StartWifi),
            Phase::Scanning => Some(Core0Action::StartWifiScan),
            Phase::Connecting => self
                .matched_configs
                .get(self.current_config_idx)
                .map(|c| Core0Action::ConnectWifi(c.clone())),
            Phase::SettingTimezone => Some(Core0Action::SetTimezone),
            Phase::SyncingNtp => Some(Core0Action::StartNtp),
            Phase::WaitingNtp => Some(Core0Action::CheckNtpStatus),
            Phase::Disconnecting => Some(Core0Action::StopWifi),
            _ => None,
        };
        Snapshot::QuickSync(QuickSyncSnapshot {
            phase: self.phase.clone(),
            matched_configs_len: self.matched_configs.len(),
            current_config_idx: self.current_config_idx,
            status_text: self.status_text.clone(),
            pending_action,
            lang: shared.lang,
        })
    }
}

// ---- Snapshot ----

pub struct QuickSyncSnapshot {
    pub phase: Phase,
    pub matched_configs_len: usize,
    pub current_config_idx: usize,
    pub status_text: String,
    pub pending_action: Option<Core0Action>,
    pub lang: InputLanguage,
}

impl QuickSyncSnapshot {
    pub fn draw(&self, ui: &mut CardworderUi) {
        let l = self.lang;
        let mut y = TOP_BAR_HEIGHT as i32 + 10;

        // Title
        ui.draw_text_oneline(
            t("Quick Sync", "Быстрая синхронизация", l),
            CardFont::Large,
            ThemeColor::Selected,
            Point::new(4, y),
            VerticalPosition::Top,
        );
        y += ui.font_height(CardFont::Large) as i32 + 8;

        // Status
        ui.draw_text_oneline(
            self.status_text.as_str(),
            CardFont::Medium,
            ThemeColor::Text,
            Point::new(4, y),
            VerticalPosition::Top,
        );
        y += ui.font_height(CardFont::Medium) as i32 + 8;

        // Progress info when connecting to multiple networks
        if matches!(self.phase, Phase::Connecting) && self.matched_configs_len > 1 {
            let mut prog = heapless::String::<32>::new();
            let _ = write!(
                prog,
                "({}/{})",
                self.current_config_idx + 1,
                self.matched_configs_len
            );
            ui.draw_text_oneline(
                prog.as_str(),
                CardFont::Small,
                ThemeColor::Text,
                Point::new(4, y),
                VerticalPosition::Top,
            );
            y += ui.font_height(CardFont::Small) as i32 + 4;
        }

        // Hint
        if matches!(self.phase, Phase::Error | Phase::NoConfigs) {
            ui.draw_text_oneline(
                t("Press Enter/Esc", "Нажмите Enter/Esc", l),
                CardFont::Small,
                ThemeColor::Text,
                Point::new(4, y),
                VerticalPosition::Top,
            );
        } else {
            ui.draw_text_oneline(
                t("Press Esc to cancel", "Esc для отмены", l),
                CardFont::Small,
                ThemeColor::Text,
                Point::new(4, y),
                VerticalPosition::Top,
            );
        }
    }
}
