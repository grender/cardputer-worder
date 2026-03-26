use embedded_graphics::pixelcolor::Rgb565;
use embedded_graphics::prelude::RgbColor;

use crate::screen::{Screen, Snapshot};
use crate::screens::main_menu::MainMenuScreen;
use crate::cardputer_hal::wifi::wifi::WifiConfig;
use crate::types::{Command, Core0Action, Core0Result, Msg, SharedState};
use crate::ui::cardworder_ui::CardworderUi;

#[derive(Clone)]
enum StartPhase {
    SettingTimezone,
    CreatingWifiFile,
    LoadingWifiConfig,
    ConnectingWifi,
    StartingNtp,
    AwaitingNtp,
    StoppingWifi,
    Done,
}

pub struct StartScreen {
    phase: StartPhase,
    wifi_config: Option<WifiConfig>,
}

impl StartScreen {
    pub fn new() -> Self {
        Self {
            phase: StartPhase::SettingTimezone,
            wifi_config: None,
        }
    }

    fn status_text(&self) -> &'static str {
        match self.phase {
            StartPhase::SettingTimezone => "Setting timezone...",
            StartPhase::CreatingWifiFile => "Creating WiFi config...",
            StartPhase::LoadingWifiConfig => "Loading WiFi config...",
            StartPhase::ConnectingWifi => "Connecting WiFi...",
            StartPhase::StartingNtp => "Starting NTP...",
            StartPhase::AwaitingNtp => "Awaiting NTP sync...",
            StartPhase::StoppingWifi => "Stopping WiFi...",
            StartPhase::Done => "Done!",
        }
    }

    fn current_action(&self) -> Option<Core0Action> {
        match &self.phase {
            StartPhase::SettingTimezone => Some(Core0Action::SetTimezone),
            StartPhase::CreatingWifiFile => Some(Core0Action::CreateWifiFileIfNotExists {
                ssid: heapless::String::try_from("John24").unwrap(),
                password: heapless::String::try_from("52525252").unwrap(),
            }),
            StartPhase::LoadingWifiConfig => Some(Core0Action::LoadWifiConfig),
            StartPhase::ConnectingWifi => self.wifi_config.clone().map(Core0Action::ConnectWifi),
            StartPhase::StartingNtp => Some(Core0Action::StartNtp),
            StartPhase::AwaitingNtp => Some(Core0Action::CheckNtpStatus),
            StartPhase::StoppingWifi => Some(Core0Action::StopWifi),
            StartPhase::Done => None,
        }
    }
}

impl Screen for StartScreen {
    fn on_mount(
        &mut self,
        _shared: &SharedState,
        _state_tx: &std::sync::mpsc::Sender<Snapshot>,
    ) -> Command {
        self.phase = StartPhase::SettingTimezone;
        Command::None
    }

    fn handle_msg(&mut self, msg: Msg, _shared: &SharedState) -> Command {
        match msg {
            Msg::Core0Result(result) => {
                match result {
                    Core0Result::TimezoneSet => {
                        self.phase = StartPhase::CreatingWifiFile;
                    }
                    Core0Result::WifiFileCreated => {
                        self.phase = StartPhase::LoadingWifiConfig;
                    }
                    Core0Result::WifiConfigLoaded(config) => {
                        self.wifi_config = Some(config);
                        self.phase = StartPhase::ConnectingWifi;
                    }
                    Core0Result::WifiConnected { .. } => {
                        self.phase = StartPhase::StartingNtp;
                    }
                    Core0Result::NtpStarted => {
                        self.phase = StartPhase::AwaitingNtp;
                    }
                    Core0Result::NtpSynced(done) => {
                        if done {
                            self.phase = StartPhase::StoppingWifi;
                        }
                        // else stay in AwaitingNtp, Core 0 will check again
                    }
                    Core0Result::WifiStopped => {
                        self.phase = StartPhase::Done;
                        return Command::SwitchTo(Box::new(MainMenuScreen::default()));
                    }
                    Core0Result::Error(msg) => {
                        log::error!("StartScreen: Core0 error: {}", msg);
                        return Command::SwitchTo(Box::new(MainMenuScreen::default()));
                    }
                    _ => {} // ignore unrelated results
                }
                Command::None
            }
            _ => Command::None,
        }
    }

    fn snapshot(&self, _shared: &SharedState) -> Snapshot {
        Snapshot::Start(StartSnapshot {
            status_text: self.status_text(),
            pending_action: self.current_action(),
        })
    }
}

// ---- Snapshot ----

pub struct StartSnapshot {
    pub status_text: &'static str,
    pub pending_action: Option<Core0Action>,
}

impl StartSnapshot {
    pub fn draw(&self, ui: &mut CardworderUi) {
        ui.draw_starting_line_text(self.status_text, Rgb565::BLACK, Rgb565::WHITE);
    }
}
