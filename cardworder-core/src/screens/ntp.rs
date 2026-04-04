use u8g2_fonts::types::VerticalPosition;

use crate::input::keyboard::{InputLanguage, PressedSymbol};
use crate::input::keyboard_io::KeyEvent;
use crate::screen::{Screen, Snapshot};
use crate::screens::main_menu::MainMenuScreen;
use crate::types::{Command, Core0Action, Core0Result, Msg, SharedState};
use crate::ui::cardworder_ui::{CardFont, CardworderUi, ThemeColor, TOP_BAR_HEIGHT};
use crate::ui::framebuffer::CardworderFB;

#[derive(Clone)]
enum NtpPhase {
    NotConnected,
    SettingTimezone,
    StartingNtp,
    AwaitingNtp,
    Done,
    Error,
}

fn ts(en: &str, ru: &str, lang: InputLanguage) -> String {
    match lang { InputLanguage::En => en, InputLanguage::Ru => ru }.to_string()
}

pub struct NtpScreen {
    phase: NtpPhase,
    status_text: String,
}

impl NtpScreen {
    pub fn new() -> Self {
        Self {
            phase: NtpPhase::SettingTimezone,
            status_text: "Initializing...".to_string(),
        }
    }
}

impl Screen for NtpScreen {
    fn on_mount(
        &mut self,
        shared: &SharedState,
        _state_tx: &std::sync::mpsc::Sender<Snapshot>,
    ) -> Command {
        if !shared.wifi_connected {
            self.phase = NtpPhase::NotConnected;
            self.status_text = ts("WiFi not connected!", "WiFi не подключен!", shared.lang);
        } else {
            self.phase = NtpPhase::SettingTimezone;
            self.status_text = ts("Setting timezone...", "Установка часового пояса...", shared.lang);
        }
        Command::None
    }

    fn handle_msg(&mut self, msg: Msg, shared: &SharedState) -> Command {
        match msg {
            Msg::Core0Result(result) => {
                match result {
                    Core0Result::TimezoneSet => {
                        self.phase = NtpPhase::StartingNtp;
                        self.status_text = ts("Starting NTP...", "Запуск NTP...", shared.lang);
                    }
                    Core0Result::NtpStarted => {
                        self.phase = NtpPhase::AwaitingNtp;
                        self.status_text = ts("Syncing time...", "Синхронизация...", shared.lang);
                    }
                    Core0Result::NtpSynced(done) => {
                        if done {
                            self.phase = NtpPhase::Done;
                            self.status_text = ts("Time synced!", "Время синхронизировано!", shared.lang);
                        }
                    }
                    Core0Result::Error(msg) => {
                        log::error!("NTP error: {}", msg);
                        self.phase = NtpPhase::Error;
                        self.status_text = msg;
                    }
                    _ => {}
                }
                Command::None
            }
            Msg::Key(key_msg) => match key_msg.pressed {
                Some((KeyEvent::Pressed, PressedSymbol::Esc))
                | Some((KeyEvent::Pressed, PressedSymbol::Enter)) => {
                    match self.phase {
                        NtpPhase::NotConnected | NtpPhase::Done | NtpPhase::Error => {
                            Command::SwitchTo(Box::new(MainMenuScreen::default()))
                        }
                        _ => Command::None,
                    }
                }
                _ => Command::None,
            },
            _ => Command::None,
        }
    }

    fn snapshot(&self, shared: &SharedState) -> Snapshot {
        let pending_action = match self.phase {
            NtpPhase::SettingTimezone => Some(Core0Action::SetTimezone),
            NtpPhase::StartingNtp => Some(Core0Action::StartNtp),
            NtpPhase::AwaitingNtp => Some(Core0Action::CheckNtpStatus),
            _ => None,
        };

        Snapshot::Ntp(NtpSnapshot {
            status_text: self.status_text.clone(),
            phase: self.phase.clone(),
            pending_action,
            lang: shared.lang,
        })
    }
}

// ---- Snapshot ----

pub struct NtpSnapshot {
    pub status_text: String,
    pub phase: NtpPhase,
    pub pending_action: Option<Core0Action>,
    pub lang: InputLanguage,
}

fn t(en: &'static str, ru: &'static str, lang: InputLanguage) -> &'static str {
    match lang { InputLanguage::En => en, InputLanguage::Ru => ru }
}

impl NtpSnapshot {
    pub fn draw<FB: CardworderFB>(&self, ui: &mut CardworderUi<FB>) {
        use embedded_graphics::prelude::Point;

        let font = CardFont::Medium;
        let mut y = TOP_BAR_HEIGHT as i32 + 10;

        let l = self.lang;
        ui.draw_text_oneline(t("NTP Time Sync", "Синхр. времени NTP", l), CardFont::Large, ThemeColor::Selected, Point::new(4, y), VerticalPosition::Top);
        y += ui.font_height(CardFont::Large) as i32 + 8;

        let color = match self.phase {
            NtpPhase::NotConnected | NtpPhase::Error => ThemeColor::Error,
            NtpPhase::Done => ThemeColor::Color(embedded_graphics::pixelcolor::Rgb565::new(0, 63, 0)),
            _ => ThemeColor::Text,
        };
        ui.draw_text_oneline(self.status_text.as_str(), font, color, Point::new(4, y), VerticalPosition::Top);
        y += ui.font_height(font) as i32 + 4;

        match self.phase {
            NtpPhase::NotConnected => {
                ui.draw_text_oneline(t("Connect to WiFi first,", "Сначала подключите WiFi,", l), CardFont::Small, ThemeColor::Text, Point::new(4, y), VerticalPosition::Top);
                y += ui.font_height(CardFont::Small) as i32 + 2;
                ui.draw_text_oneline(t("then try again.", "затем попробуйте снова.", l), CardFont::Small, ThemeColor::Text, Point::new(4, y), VerticalPosition::Top);
                y += ui.font_height(CardFont::Small) as i32 + 8;
                ui.draw_text_oneline(t("<- Back (Enter/Esc)", "<- Назад (Enter/Esc)", l), font, ThemeColor::Selected, Point::new(4, y), VerticalPosition::Top);
            }
            NtpPhase::Done => {
                ui.draw_text_oneline(t("<- Back (Enter/Esc)", "<- Назад (Enter/Esc)", l), font, ThemeColor::Selected, Point::new(4, y), VerticalPosition::Top);
            }
            NtpPhase::Error => {
                y += 4;
                ui.draw_text_oneline(t("<- Back (Enter/Esc)", "<- Назад (Enter/Esc)", l), font, ThemeColor::Selected, Point::new(4, y), VerticalPosition::Top);
            }
            _ => {
                ui.draw_text_oneline(t("Please wait...", "Подождите...", l), CardFont::Small, ThemeColor::Text, Point::new(4, y), VerticalPosition::Top);
            }
        }
    }
}
