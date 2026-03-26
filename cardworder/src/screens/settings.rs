use u8g2_fonts::types::VerticalPosition;


use crate::cardputer_hal::input::keyboard::PressedSymbol;
use crate::cardputer_hal::input::keyboard_io::KeyEvent;
use crate::screen::{Screen, Snapshot};
use crate::screens::main_menu::MainMenuScreen;
use crate::types::{Command, Msg, SharedState};
use crate::ui::cardworder_ui::{CardFont, CardworderUi, ThemeColor, TOP_BAR_HEIGHT};
use crate::ui::elements::{UiLineElement, UiLineType};
use crate::ui::render::{compose_scrolled_form, render_visible_lines};

const NUM_FOCUSABLE: usize = 3; // ssid, password, back button

pub struct SettingsScreen {
    ssid: heapless::String<64>,
    password: heapless::String<64>,
    ssid_cursor: usize,
    password_cursor: usize,
    focused_idx: usize,
}

impl SettingsScreen {
    pub fn new() -> Self {
        Self {
            ssid: heapless::String::new(),
            password: heapless::String::new(),
            ssid_cursor: 0,
            password_cursor: 0,
            focused_idx: 0,
        }
    }

    fn active_value_mut(&mut self) -> Option<(&mut heapless::String<64>, &mut usize)> {
        match self.focused_idx {
            0 => Some((&mut self.ssid, &mut self.ssid_cursor)),
            1 => Some((&mut self.password, &mut self.password_cursor)),
            _ => None,
        }
    }

    fn handle_input_key(&mut self, symbol: PressedSymbol) {
        if let Some((value, cursor)) = self.active_value_mut() {
            match symbol {
                PressedSymbol::Char(c) => {
                    if value.len() < 64 {
                        // Insert char at cursor position
                        let pos = byte_pos_of_char(value.as_str(), *cursor);
                        let mut new_val = heapless::String::<64>::new();
                        let _ = new_val.push_str(&value.as_str()[..pos]);
                        let _ = new_val.push(c);
                        let _ = new_val.push_str(&value.as_str()[pos..]);
                        *value = new_val;
                        *cursor += 1;
                    }
                }
                PressedSymbol::Backspace => {
                    if *cursor > 0 {
                        let prev_pos = byte_pos_of_char(value.as_str(), *cursor - 1);
                        let cur_pos = byte_pos_of_char(value.as_str(), *cursor);
                        let mut new_val = heapless::String::<64>::new();
                        let _ = new_val.push_str(&value.as_str()[..prev_pos]);
                        let _ = new_val.push_str(&value.as_str()[cur_pos..]);
                        *value = new_val;
                        *cursor -= 1;
                    }
                }
                PressedSymbol::Del => {
                    let char_count = value.chars().count();
                    if *cursor < char_count {
                        let cur_pos = byte_pos_of_char(value.as_str(), *cursor);
                        let next_pos = byte_pos_of_char(value.as_str(), *cursor + 1);
                        let mut new_val = heapless::String::<64>::new();
                        let _ = new_val.push_str(&value.as_str()[..cur_pos]);
                        let _ = new_val.push_str(&value.as_str()[next_pos..]);
                        *value = new_val;
                    }
                }
                PressedSymbol::ArrowLeft => {
                    if *cursor > 0 {
                        *cursor -= 1;
                    }
                }
                PressedSymbol::ArrowRight => {
                    let char_count = value.chars().count();
                    if *cursor < char_count {
                        *cursor += 1;
                    }
                }
                _ => {}
            }
        }
    }
}

/// Get byte offset of the nth character in a UTF-8 string.
fn byte_pos_of_char(s: &str, n: usize) -> usize {
    s.char_indices()
        .nth(n)
        .map(|(i, _)| i)
        .unwrap_or(s.len())
}

impl Screen for SettingsScreen {
    fn handle_msg(&mut self, msg: Msg, _shared: &SharedState) -> Command {
        match msg {
            Msg::Key(key_msg) => match key_msg.pressed {
                Some((KeyEvent::Pressed, PressedSymbol::Esc)) => {
                    Command::SwitchTo(Box::new(MainMenuScreen::default()))
                }
                Some((KeyEvent::Pressed, PressedSymbol::ArrowDown)) => {
                    self.focused_idx = (self.focused_idx + 1) % NUM_FOCUSABLE;
                    Command::None
                }
                Some((KeyEvent::Pressed, PressedSymbol::ArrowUp)) => {
                    self.focused_idx =
                        (self.focused_idx + NUM_FOCUSABLE - 1) % NUM_FOCUSABLE;
                    Command::None
                }
                Some((KeyEvent::Pressed, PressedSymbol::Enter)) => {
                    if self.focused_idx == NUM_FOCUSABLE - 1 {
                        // Back button
                        Command::SwitchTo(Box::new(MainMenuScreen::default()))
                    } else {
                        Command::None
                    }
                }
                Some((KeyEvent::Pressed, symbol)) => {
                    self.handle_input_key(symbol);
                    Command::None
                }
                _ => Command::None,
            },
            _ => Command::None,
        }
    }

    fn snapshot(&self, shared: &SharedState) -> Snapshot {
        Snapshot::Settings(SettingsSnapshot {
            ssid: self.ssid.clone(),
            password: self.password.clone(),
            ssid_cursor: self.ssid_cursor,
            password_cursor: self.password_cursor,
            focused_idx: self.focused_idx,
            lang: shared.lang,
        })
    }
}

// ---- Snapshot ----

pub struct SettingsSnapshot {
    pub ssid: heapless::String<64>,
    pub password: heapless::String<64>,
    pub ssid_cursor: usize,
    pub password_cursor: usize,
    pub focused_idx: usize,
    pub lang: crate::cardputer_hal::input::keyboard::InputLanguage,
}

impl SettingsSnapshot {
    pub fn draw(&self, ui: &mut CardworderUi) {
        const SCREEN_HEIGHT: u32 = 135;
        let viewport_height = SCREEN_HEIGHT - TOP_BAR_HEIGHT;

        let back_color = if self.focused_idx == NUM_FOCUSABLE - 1 {
            ThemeColor::Selected
        } else {
            ThemeColor::Text
        };

        use crate::cardputer_hal::input::keyboard::InputLanguage;
        let l = self.lang;
        let t = |en: &'static str, ru: &'static str| -> &'static str {
            match l { InputLanguage::En => en, InputLanguage::Ru => ru }
        };

        let lines: Vec<UiLineType> = vec![
            UiLineType::InputField {
                label: t("WiFi SSID", "WiFi SSID"),
                value: self.ssid.clone(),
                cursor_pos: self.ssid_cursor,
                focused: self.focused_idx == 0,
            },
            // Password input
            UiLineType::InputField {
                label: t("WiFi Password", "Пароль WiFi"),
                value: self.password.clone(),
                cursor_pos: self.password_cursor,
                focused: self.focused_idx == 1,
            },
            // Spacer
            UiLineType::Spacer(8),
            // Back button
            UiLineType::Elements(vec![
                UiLineElement::Text(
                    t("<- Back", "<- Назад"),
                    CardFont::Medium,
                    VerticalPosition::Top,
                    back_color,
                ),
            ]),
        ];

        // Use focused_idx mapped to line index for scroll
        let scroll_target = match self.focused_idx {
            0 => 0, // ssid
            1 => 1, // password
            _ => 3, // back button
        };

        let composed = compose_scrolled_form(
            &lines,
            scroll_target,
            viewport_height,
            TOP_BAR_HEIGHT as i32,
            ui,
        );
        render_visible_lines(&composed, &lines, ui);
    }
}
