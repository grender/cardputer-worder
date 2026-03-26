use u8g2_fonts::types::VerticalPosition;

use crate::cardputer_hal::cardputer_hal::CardputerHal;
use crate::cardputer_hal::input::keyboard::{InputLanguage, PressedSymbol};
use crate::cardputer_hal::input::keyboard_io::{KeyEvent, Scancode};
use crate::screen::{Renderable, Screen};
use crate::screens::start::StartScreen;
use crate::screens::system_info::SystemInfoScreen;
use crate::types::{Command, KeyMsg, Msg, SharedState};
use crate::ui::cardworder_ui::{CardFont, CardworderUi, ThemeColor, TOP_BAR_HEIGHT};
use crate::ui::elements::{UiLineElement, UiLineType};
use crate::ui::render::{compose_scrolled_form, render_visible_lines};

enum MainMenuOption {
    ConnectWifi,
    UpdateNtp,
    SystemInfo,
    AdditionalInfo,
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
                MainMenuOption::ConnectWifi,
                MainMenuOption::UpdateNtp,
                MainMenuOption::SystemInfo,
                MainMenuOption::AdditionalInfo,
            ],
            current_item_idx: 0,
            lang: InputLanguage::En,
            show_fps: false,
        }
    }
}

fn get_option_text(option: &MainMenuOption, lang: InputLanguage) -> &'static str {
    match (lang, option) {
        (InputLanguage::En, MainMenuOption::ConnectWifi) => "Connect Wi-Fi",
        (InputLanguage::En, MainMenuOption::UpdateNtp) => "Update time by NTP",
        (InputLanguage::Ru, MainMenuOption::ConnectWifi) => "Подключить Wi-Fi",
        (InputLanguage::Ru, MainMenuOption::UpdateNtp) => "Обновить время по NTP",
        (InputLanguage::En, MainMenuOption::SystemInfo) => "System Info",
        (InputLanguage::Ru, MainMenuOption::SystemInfo) => "Системная информация",
        (InputLanguage::En, MainMenuOption::AdditionalInfo) => "Additional info",
        (InputLanguage::Ru, MainMenuOption::AdditionalInfo) => "Дополнительная информация",
    }
}

fn get_option_icon(option: &MainMenuOption) -> char {
    match option {
        MainMenuOption::ConnectWifi => '\u{25A}',
        MainMenuOption::UpdateNtp => '\u{158}',
        MainMenuOption::SystemInfo => '\u{15e}',
        MainMenuOption::AdditionalInfo => '\u{1d5}',
    }
}

impl Screen for MainMenuScreen {
    fn handle_msg(&mut self, msg: Msg, _hal: &mut CardputerHal<'_>, _shared: &SharedState) -> Command {
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
                            MainMenuOption::ConnectWifi => {
                                return Command::SwitchTo(Box::new(StartScreen::new()));
                            }
                            MainMenuOption::SystemInfo => {
                                return Command::SwitchTo(Box::new(SystemInfoScreen::new()));
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

    fn snapshot(&self) -> Box<dyn Renderable> {
        let items: Vec<MainMenuSnapshotItem> = self
            .options
            .iter()
            .map(|o| MainMenuSnapshotItem {
                icon: get_option_icon(o),
                text: get_option_text(o, self.lang),
            })
            .collect();

        Box::new(MainMenuSnapshot {
            items,
            selected_idx: self.current_item_idx,
            show_fps: self.show_fps,
        })
    }
}

// ---- Snapshot (sent to Core 0) ----

struct MainMenuSnapshotItem {
    icon: char,
    text: &'static str,
}

struct MainMenuSnapshot {
    items: Vec<MainMenuSnapshotItem>,
    selected_idx: usize,
    show_fps: bool,
}

// SAFETY: MainMenuSnapshot only contains data types (no pointers to non-Send data)
unsafe impl Send for MainMenuSnapshot {}

impl Renderable for MainMenuSnapshot {
    fn draw(&self, ui: &mut CardworderUi) {
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
