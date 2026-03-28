use u8g2_fonts::types::VerticalPosition;


use crate::cardputer_hal::input::keyboard::{InputLanguage, PressedSymbol};
use crate::cardputer_hal::input::keyboard_io::{KeyEvent, Scancode};
use crate::screen::{Screen, Snapshot};
use crate::screens::add_word::AddWordScreen;
use crate::screens::review::ReviewScreen;
use crate::screens::statistics::StatisticsScreen;
use crate::screens::wifi_config::WifiConfigScreen;
use crate::screens::ntp::NtpScreen;
use crate::screens::quick_sync::QuickSyncScreen;
use crate::screens::system_info::SystemInfoScreen;
use crate::types::{Command, Msg, SharedState};
use crate::ui::cardworder_ui::{CardFont, CardworderUi, ThemeColor, TOP_BAR_HEIGHT};
use crate::ui::elements::{UiLineElement, UiLineType};
use crate::ui::render::{compose_scrolled_form, render_visible_lines};

enum MainMenuOption {
    ReviewWords,
    AddWord,
    Statistics,
    QuickSync,
    ConnectWifi,
    UpdateNtp,
    SystemInfo,
}

pub struct MainMenuScreen {
    options: Vec<MainMenuOption>,
    current_item_idx: usize,
    lang: InputLanguage,
    show_fps: bool,
}

impl Default for MainMenuScreen {
    fn default() -> Self {
        Self {
            options: vec![
                MainMenuOption::ReviewWords,
                MainMenuOption::AddWord,
                MainMenuOption::Statistics,
                MainMenuOption::QuickSync,
                MainMenuOption::ConnectWifi,
                MainMenuOption::UpdateNtp,
                MainMenuOption::SystemInfo,
            ],
            current_item_idx: 0,
            lang: InputLanguage::En,
            show_fps: false,
        }
    }
}

fn get_option_text(option: &MainMenuOption, lang: InputLanguage) -> &'static str {
    match (lang, option) {
        (InputLanguage::En, MainMenuOption::ReviewWords) => "Review Words",
        (InputLanguage::Ru, MainMenuOption::ReviewWords) => "Повторение слов",
        (InputLanguage::En, MainMenuOption::AddWord) => "Add Word",
        (InputLanguage::Ru, MainMenuOption::AddWord) => "Добавить слово",
        (InputLanguage::En, MainMenuOption::Statistics) => "Statistics",
        (InputLanguage::Ru, MainMenuOption::Statistics) => "Статистика",
        (InputLanguage::En, MainMenuOption::QuickSync) => "Quick Sync",
        (InputLanguage::Ru, MainMenuOption::QuickSync) => "Быстрая синхронизация",
        (InputLanguage::En, MainMenuOption::ConnectWifi) => "Connect Wi-Fi",
        (InputLanguage::Ru, MainMenuOption::ConnectWifi) => "Подключить Wi-Fi",
        (InputLanguage::En, MainMenuOption::UpdateNtp) => "Update NTP",
        (InputLanguage::Ru, MainMenuOption::UpdateNtp) => "Обновить время по NTP",
        (InputLanguage::En, MainMenuOption::SystemInfo) => "System Info",
        (InputLanguage::Ru, MainMenuOption::SystemInfo) => "Системная информация",
    }
}

fn get_option_icon(option: &MainMenuOption) -> char {
    match option {
        MainMenuOption::ReviewWords => '\u{158}',
        MainMenuOption::AddWord => '\u{1d5}',
        MainMenuOption::Statistics => '\u{15e}',
        MainMenuOption::QuickSync => '\u{158}',
        MainMenuOption::ConnectWifi => '\u{25A}',
        MainMenuOption::UpdateNtp => '\u{158}',
        MainMenuOption::SystemInfo => '\u{15e}',
    }
}

impl Screen for MainMenuScreen {
    fn handle_msg(&mut self, msg: Msg, _shared: &SharedState) -> Command {
        match msg {
            Msg::Key(key_msg) => {
                self.lang = key_msg.input_state.lang;

                // Opt+F toggles FPS
                if let (true, Some((KeyEvent::Pressed, Scancode::F))) =
                    (key_msg.input_state.opt_pressed, key_msg.key)
                {
                    self.show_fps = !self.show_fps;
                }

                match key_msg.pressed {
                    Some((KeyEvent::Pressed, PressedSymbol::ArrowDown)) => {
                        self.current_item_idx = (self.current_item_idx + 1) % self.options.len();
                    }
                    Some((KeyEvent::Pressed, PressedSymbol::ArrowUp)) => {
                        self.current_item_idx =
                            (self.current_item_idx + self.options.len() - 1) % self.options.len();
                    }
                    Some((KeyEvent::Pressed, PressedSymbol::Enter)) => {
                        match self.options[self.current_item_idx] {
                            MainMenuOption::ReviewWords => {
                                return Command::SwitchTo(Box::new(ReviewScreen::new()));
                            }
                            MainMenuOption::AddWord => {
                                return Command::SwitchTo(Box::new(AddWordScreen::new()));
                            }
                            MainMenuOption::Statistics => {
                                return Command::SwitchTo(Box::new(StatisticsScreen::new()));
                            }
                            MainMenuOption::ConnectWifi => {
                                return Command::SwitchTo(Box::new(WifiConfigScreen::new()));
                            }
                            MainMenuOption::UpdateNtp => {
                                return Command::SwitchTo(Box::new(NtpScreen::new()));
                            }
                            MainMenuOption::SystemInfo => {
                                return Command::SwitchTo(Box::new(SystemInfoScreen::new()));
                            }
                            MainMenuOption::QuickSync => {
                                return Command::SwitchTo(Box::new(QuickSyncScreen::new()));
                            }
                            _ => {}
                        }
                    }
                    _ => {}
                }
                Command::None
            }
            _ => Command::None,
        }
    }

    fn snapshot(&self, _shared: &SharedState) -> Snapshot {
        let items: Vec<MainMenuSnapshotItem> = self
            .options
            .iter()
            .map(|o| MainMenuSnapshotItem {
                icon: get_option_icon(o),
                text: get_option_text(o, self.lang),
            })
            .collect();

        Snapshot::MainMenu(MainMenuSnapshot {
            items,
            selected_idx: self.current_item_idx,
            show_fps: self.show_fps,
        })
    }
}

// ---- Snapshot (sent to Core 0) ----

pub struct MainMenuSnapshotItem {
    pub icon: char,
    pub text: &'static str,
}

pub struct MainMenuSnapshot {
    pub items: Vec<MainMenuSnapshotItem>,
    pub selected_idx: usize,
    pub show_fps: bool,
}

impl MainMenuSnapshot {
    pub fn draw(&self, ui: &mut CardworderUi) {
        const SCREEN_HEIGHT: u32 = 135;
        let viewport_height = SCREEN_HEIGHT - TOP_BAR_HEIGHT;

        let lines: Vec<UiLineType> = self
            .items
            .iter()
            .enumerate()
            .map(|(idx, item)| {
                let color = if idx == self.selected_idx {
                    ThemeColor::Selected
                } else {
                    ThemeColor::Text
                };
                UiLineType::Elements(vec![
                    UiLineElement::Icon(item.icon, CardFont::IconsHuge, color),
                    UiLineElement::Spacer(8),
                    UiLineElement::Text(
                        item.text,
                        CardFont::Medium,
                        VerticalPosition::Center,
                        color,
                    ),
                ])
            })
            .collect();

        let composed = compose_scrolled_form(
            &lines,
            self.selected_idx,
            viewport_height,
            TOP_BAR_HEIGHT as i32,
            ui,
        );
        render_visible_lines(&composed, &lines, ui);
        ui.show_fps = self.show_fps;
    }
}
