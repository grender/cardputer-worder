use embedded_graphics::prelude::Point;
use u8g2_fonts::types::VerticalPosition;

use crate::cardputer_hal::input::keyboard::{InputLanguage, PressedSymbol};
use crate::cardputer_hal::input::keyboard_io::KeyEvent;
use crate::screen::{Screen, Snapshot};
use crate::screens::main_menu::MainMenuScreen;
use crate::types::{Command, Core0Action, Core0Result, Msg, SharedState};
use crate::ui::cardworder_ui::{CardFont, CardworderUi, ThemeColor, TOP_BAR_HEIGHT};
use crate::ui::elements::{UiLineElement, UiLineType};
use crate::ui::render::{compose_scrolled_form, render_visible_lines};
use fsrs_core::{PairsFile, WordPair};

#[derive(Clone)]
enum Phase {
    Loading,
    Editing,
    Saving,
    Error,
}

fn t(en: &'static str, ru: &'static str, lang: InputLanguage) -> &'static str {
    match lang {
        InputLanguage::En => en,
        InputLanguage::Ru => ru,
    }
}

fn byte_pos_of_char(s: &str, n: usize) -> usize {
    s.char_indices().nth(n).map(|(i, _)| i).unwrap_or(s.len())
}

pub struct AddWordScreen {
    phase: Phase,
    pairs_file: Option<PairsFile>,
    en_value: heapless::String<64>,
    ru_value: heapless::String<64>,
    en_cursor: usize,
    ru_cursor: usize,
    focused_field: usize, // 0=English, 1=Russian, 2=Save, 3=Back
    status_text: String,
}

impl Default for AddWordScreen {
    fn default() -> Self {
        Self {
            phase: Phase::Loading,
            pairs_file: None,
            en_value: heapless::String::new(),
            ru_value: heapless::String::new(),
            en_cursor: 0,
            ru_cursor: 0,
            focused_field: 0,
            status_text: String::new(),
        }
    }
}

impl AddWordScreen {
    pub fn new() -> Self {
        Self::default()
    }

    fn handle_input_key(&mut self, symbol: PressedSymbol) {
        let (value, cursor) = match self.focused_field {
            0 => (&mut self.en_value, &mut self.en_cursor),
            1 => (&mut self.ru_value, &mut self.ru_cursor),
            _ => return,
        };
        match symbol {
            PressedSymbol::Char(c) => {
                if value.len() + c.len_utf8() <= 64 {
                    let pos = byte_pos_of_char(value.as_str(), *cursor);
                    let mut nv = heapless::String::<64>::new();
                    let _ = nv.push_str(&value.as_str()[..pos]);
                    let _ = nv.push(c);
                    let _ = nv.push_str(&value.as_str()[pos..]);
                    *value = nv;
                    *cursor += 1;
                }
            }
            PressedSymbol::Backspace => {
                if *cursor > 0 {
                    let p = byte_pos_of_char(value.as_str(), *cursor - 1);
                    let c = byte_pos_of_char(value.as_str(), *cursor);
                    let mut nv = heapless::String::<64>::new();
                    let _ = nv.push_str(&value.as_str()[..p]);
                    let _ = nv.push_str(&value.as_str()[c..]);
                    *value = nv;
                    *cursor -= 1;
                }
            }
            PressedSymbol::Del => {
                let char_count = value.chars().count();
                if *cursor < char_count {
                    let p = byte_pos_of_char(value.as_str(), *cursor);
                    let c = byte_pos_of_char(value.as_str(), *cursor + 1);
                    let mut nv = heapless::String::<64>::new();
                    let _ = nv.push_str(&value.as_str()[..p]);
                    let _ = nv.push_str(&value.as_str()[c..]);
                    *value = nv;
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

    fn clear_fields(&mut self) {
        self.en_value = heapless::String::new();
        self.ru_value = heapless::String::new();
        self.en_cursor = 0;
        self.ru_cursor = 0;
        self.focused_field = 0;
    }
}

impl Screen for AddWordScreen {
    fn on_mount(
        &mut self,
        _shared: &SharedState,
        _state_tx: &std::sync::mpsc::Sender<Snapshot>,
    ) -> Command {
        self.phase = Phase::Loading;
        Command::None
    }

    fn handle_msg(&mut self, msg: Msg, shared: &SharedState) -> Command {
        let l = shared.lang;
        match msg {
            Msg::Core0Result(result) => {
                match result {
                    Core0Result::PairsLoaded(pairs_file) => {
                        self.pairs_file = Some(pairs_file);
                        self.phase = Phase::Editing;
                        self.status_text = String::new();
                    }
                    Core0Result::PairsSaved => {
                        self.clear_fields();
                        self.phase = Phase::Editing;
                        self.status_text = t("Saved!", "Сохранено!", l).to_string();
                    }
                    Core0Result::Error(msg) => {
                        self.status_text = msg;
                        self.phase = Phase::Error;
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
                    Some((KeyEvent::Pressed, PressedSymbol::ArrowDown)) => {
                        if matches!(self.phase, Phase::Editing) {
                            self.focused_field = (self.focused_field + 1) % 4;
                        }
                    }
                    Some((KeyEvent::Pressed, PressedSymbol::ArrowUp)) => {
                        if matches!(self.phase, Phase::Editing) {
                            self.focused_field = (self.focused_field + 3) % 4;
                        }
                    }
                    Some((KeyEvent::Pressed, PressedSymbol::Enter)) => {
                        match self.phase {
                            Phase::Editing => {
                                match self.focused_field {
                                    2 => {
                                        // Save
                                        if self.en_value.is_empty() || self.ru_value.is_empty() {
                                            self.status_text = t(
                                                "Both fields required",
                                                "Заполните оба поля",
                                                l,
                                            )
                                            .to_string();
                                        } else if let Some(ref mut pf) = self.pairs_file {
                                            let id = pf.next_id;
                                            let pair = WordPair::new(
                                                id,
                                                self.en_value.as_str().to_string(),
                                                self.ru_value.as_str().to_string(),
                                            );
                                            pf.next_id += 1;
                                            pf.pairs.push(pair);
                                            self.phase = Phase::Saving;
                                            self.status_text =
                                                t("Saving...", "Сохранение...", l).to_string();
                                        }
                                    }
                                    3 => {
                                        // Back
                                        return Command::SwitchTo(Box::new(
                                            MainMenuScreen::default(),
                                        ));
                                    }
                                    _ => {
                                        // Enter on input fields moves to next field
                                        self.focused_field = (self.focused_field + 1) % 4;
                                    }
                                }
                            }
                            Phase::Error => {
                                // Go back to editing on error
                                self.phase = Phase::Editing;
                                self.status_text = String::new();
                            }
                            _ => {}
                        }
                    }
                    Some((KeyEvent::Pressed, symbol)) => {
                        if matches!(self.phase, Phase::Editing)
                            && (self.focused_field == 0 || self.focused_field == 1)
                        {
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
            Phase::Loading => Some(Core0Action::LoadPairs),
            Phase::Saving => self.pairs_file.as_ref().and_then(|pf| {
                postcard::to_allocvec(pf).ok().map(Core0Action::SavePairsBytes)
            }),
            _ => None,
        };
        Snapshot::AddWord(AddWordSnapshot {
            phase: self.phase.clone(),
            en_value: self.en_value.clone(),
            ru_value: self.ru_value.clone(),
            en_cursor: self.en_cursor,
            ru_cursor: self.ru_cursor,
            focused_field: self.focused_field,
            status_text: self.status_text.clone(),
            pending_action,
            lang: shared.lang,
        })
    }
}

// ---- Snapshot ----

pub struct AddWordSnapshot {
    pub phase: Phase,
    pub en_value: heapless::String<64>,
    pub ru_value: heapless::String<64>,
    pub en_cursor: usize,
    pub ru_cursor: usize,
    pub focused_field: usize,
    pub status_text: String,
    pub pending_action: Option<Core0Action>,
    pub lang: InputLanguage,
}

impl AddWordSnapshot {
    pub fn draw(&self, ui: &mut CardworderUi) {
        let font = CardFont::Medium;
        let small = CardFont::Small;
        let l = self.lang;
        let y = TOP_BAR_HEIGHT as i32 + 2;

        match self.phase {
            Phase::Loading => {
                ui.draw_text_oneline(
                    t("Loading...", "Загрузка...", l),
                    font,
                    ThemeColor::Text,
                    Point::new(4, y),
                    VerticalPosition::Top,
                );
            }
            Phase::Editing | Phase::Saving => {
                const SCREEN_HEIGHT: u32 = 135;
                let viewport_height = SCREEN_HEIGHT - TOP_BAR_HEIGHT;

                let mut lines: Vec<UiLineType> = Vec::new();

                // English input field
                lines.push(UiLineType::InputField {
                    label: t("English", "Английский", l),
                    value: self.en_value.clone(),
                    cursor_pos: self.en_cursor,
                    focused: self.focused_field == 0,
                });

                lines.push(UiLineType::Spacer(2));

                // Russian input field
                lines.push(UiLineType::InputField {
                    label: t("Russian", "Русский", l),
                    value: self.ru_value.clone(),
                    cursor_pos: self.ru_cursor,
                    focused: self.focused_field == 1,
                });

                lines.push(UiLineType::Spacer(4));

                // Save button
                let save_color = if self.focused_field == 2 {
                    ThemeColor::Selected
                } else {
                    ThemeColor::Text
                };
                lines.push(UiLineType::Elements(vec![UiLineElement::Text(
                    t("[Save]", "[Сохранить]", l),
                    font,
                    VerticalPosition::Top,
                    save_color,
                )]));

                // Back button
                let back_color = if self.focused_field == 3 {
                    ThemeColor::Selected
                } else {
                    ThemeColor::Text
                };
                lines.push(UiLineType::Elements(vec![UiLineElement::Text(
                    t("<- Back", "<- Назад", l),
                    font,
                    VerticalPosition::Top,
                    back_color,
                )]));

                // Status text
                if !self.status_text.is_empty() {
                    lines.push(UiLineType::Spacer(2));
                    let status_color = if self.status_text == t("Saved!", "Сохранено!", l) {
                        ThemeColor::Color(embedded_graphics::pixelcolor::Rgb565::new(0, 50, 0))
                    } else {
                        ThemeColor::Error
                    };
                    lines.push(UiLineType::Elements(vec![UiLineElement::Text(
                        self.status_text.as_str(),
                        small,
                        VerticalPosition::Top,
                        status_color,
                    )]));
                }

                // Determine scroll target based on focused field
                let scroll_target = match self.focused_field {
                    0 => 0,                        // English field
                    1 => 2,                        // Russian field (after spacer)
                    2 => 4,                        // Save button
                    3 => 5,                        // Back button
                    _ => 0,
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
            Phase::Error => {
                ui.draw_text_oneline(
                    t("Error:", "Ошибка:", l),
                    font,
                    ThemeColor::Error,
                    Point::new(4, y),
                    VerticalPosition::Top,
                );
                let line_h = ui.font_height(font) as i32 + 3;
                let y2 = y + line_h;
                ui.draw_text_oneline(
                    self.status_text.as_str(),
                    small,
                    ThemeColor::Text,
                    Point::new(4, y2),
                    VerticalPosition::Top,
                );
                let y3 = y2 + ui.font_height(small) as i32 + 4;
                ui.draw_text_oneline(
                    t("Press any key", "Нажмите клавишу", l),
                    small,
                    ThemeColor::Selected,
                    Point::new(4, y3),
                    VerticalPosition::Top,
                );
            }
        }
    }
}
