use core::fmt::Write;
use std::time::SystemTime;

use embedded_graphics::pixelcolor::Rgb565;

use crate::input::keyboard::{InputLanguage, PressedSymbol};
use crate::input::keyboard_io::KeyEvent;
use crate::screen::{Screen, Snapshot};
use crate::screens::main_menu::MainMenuScreen;
use crate::types::{Command, Core0Action, Core0Result, Msg, SharedState};
use crate::ui::cardworder_ui::{CardFont, CardworderUi, ThemeColor};
use crate::ui::elements::{UiLineElement, UiLineType};
use crate::ui::framebuffer::CardworderFB;
use crate::ui::render::{compose_scrolled_form, render_visible_lines};
use fsrs_core::{Direction, DueItem, FsrsRecord, BinaryDirState, Rating, FSRS_RECORD_SIZE};
use fsrs_core::models::get_timestamp_iso;

#[derive(Clone)]
enum Phase {
    NtpCheck,
    Loading,
    LoadingWord,
    ShowPrompt,
    ShowAnswer,
    SavingRating,
    SessionDone,
    Empty,
    Error(String),
}

pub struct ReviewScreen {
    phase: Phase,
    due_items: Vec<DueItem>,
    current_idx: usize,
    total_count: usize,
    current_en: String,
    current_ru: String,
    current_record_bytes: [u8; FSRS_RECORD_SIZE],
}

impl ReviewScreen {
    pub fn new() -> Self {
        Self {
            phase: Phase::NtpCheck,
            due_items: Vec::new(),
            current_idx: 0,
            total_count: 0,
            current_en: String::new(),
            current_ru: String::new(),
            current_record_bytes: [0u8; FSRS_RECORD_SIZE],
        }
    }

    fn go_main_menu() -> Command {
        Command::SwitchTo(Box::new(MainMenuScreen::default()))
    }

    fn current_prompt_answer(&self) -> Option<(&str, &str, Direction)> {
        let item = self.due_items.get(self.current_idx)?;
        let (prompt, answer) = match item.direction {
            Direction::Forward => (self.current_en.as_str(), self.current_ru.as_str()),
            Direction::Reverse => (self.current_ru.as_str(), self.current_en.as_str()),
        };
        Some((prompt, answer, item.direction))
    }

    fn apply_rating(&mut self, rating: Rating) {
        let item = &self.due_items[self.current_idx];
        let mut record = FsrsRecord::from_bytes(&self.current_record_bytes);
        let dir_state = match item.direction {
            Direction::Forward => &mut record.forward,
            Direction::Reverse => &mut record.reverse,
        };
        let mut card = dir_state.to_card();
        let _ = fsrs_core::fsrs::update_card_with_review(&mut card, rating);
        *dir_state = BinaryDirState::from_card_and_review(&card, &Some(get_timestamp_iso()));
        self.current_record_bytes = record.to_bytes();
        self.phase = Phase::SavingRating;
    }
}

fn t(en: &'static str, ru: &'static str, lang: InputLanguage) -> &'static str {
    match lang { InputLanguage::En => en, InputLanguage::Ru => ru }
}

fn is_time_synced() -> bool {
    let secs = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    secs >= 1_735_689_600
}

fn direction_label(dir: Direction) -> &'static str {
    match dir { Direction::Forward => "EN>RU", Direction::Reverse => "RU>EN" }
}

fn now_us() -> u64 {
    use std::sync::OnceLock;
    use std::time::Instant;
    static START: OnceLock<Instant> = OnceLock::new();
    START.get_or_init(Instant::now).elapsed().as_micros() as u64
}

impl Screen for ReviewScreen {
    fn on_mount(&mut self, _shared: &SharedState, _state_tx: &std::sync::mpsc::Sender<Snapshot>) -> Command {
        if !is_time_synced() {
            self.phase = Phase::NtpCheck;
        } else {
            self.phase = Phase::Loading;
        }
        Command::None
    }

    fn handle_msg(&mut self, msg: Msg, shared: &SharedState) -> Command {
        match msg {
            Msg::Key(key_msg) => match &self.phase {
                Phase::NtpCheck => match key_msg.pressed {
                    Some((KeyEvent::Pressed, PressedSymbol::Esc | PressedSymbol::Enter)) => Self::go_main_menu(),
                    _ => Command::None,
                },

                Phase::Loading | Phase::LoadingWord | Phase::SavingRating => Command::None,

                Phase::ShowPrompt => match key_msg.pressed {
                    Some((KeyEvent::Pressed, PressedSymbol::Enter)) => {
                        self.phase = Phase::ShowAnswer;
                        Command::None
                    }
                    Some((KeyEvent::Pressed, PressedSymbol::Esc)) => Self::go_main_menu(),
                    _ => Command::None,
                },

                Phase::ShowAnswer => match key_msg.pressed {
                    Some((KeyEvent::Pressed, PressedSymbol::Char(c @ '1'..='4'))) => {
                        let rating = match c {
                            '1' => Rating::Again,
                            '2' => Rating::Hard,
                            '3' => Rating::Good,
                            _ => Rating::Easy,
                        };
                        self.apply_rating(rating);
                        Command::None
                    }
                    Some((KeyEvent::Pressed, PressedSymbol::Esc)) => Self::go_main_menu(),
                    _ => Command::None,
                },

                Phase::SessionDone | Phase::Empty | Phase::Error(_) => match key_msg.pressed {
                    Some((KeyEvent::Pressed, PressedSymbol::Esc | PressedSymbol::Enter)) => Self::go_main_menu(),
                    _ => Command::None,
                },
            },

            Msg::Core0Result(result) => match result {
                Core0Result::DueItemsLoaded { forward_items, reverse_items, next_id: _ } => {
                    let mut due = match shared.lang {
                        InputLanguage::En => {
                            if !forward_items.is_empty() { forward_items } else { reverse_items }
                        }
                        InputLanguage::Ru => {
                            if !reverse_items.is_empty() { reverse_items } else { forward_items }
                        }
                    };

                    if due.is_empty() {
                        self.phase = Phase::Empty;
                    } else {
                        // Fisher-Yates shuffle
                        let mut seed = now_us();
                        for i in (1..due.len()).rev() {
                            seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
                            let j = (seed >> 33) as usize % (i + 1);
                            due.swap(i, j);
                        }
                        self.total_count = due.len();
                        self.due_items = due;
                        self.current_idx = 0;
                        self.phase = Phase::LoadingWord;
                    }
                    Command::None
                }

                Core0Result::WordTextLoaded { en, ru, record_bytes } => {
                    self.current_en = en;
                    self.current_ru = ru;
                    self.current_record_bytes = record_bytes;
                    self.phase = Phase::ShowPrompt;
                    Command::None
                }

                Core0Result::CardRated => {
                    self.current_idx += 1;
                    self.phase = Phase::SessionDone;
                    Command::None
                }

                Core0Result::CardRatedAndNextLoaded { en, ru, record_bytes } => {
                    self.current_idx += 1;
                    self.current_en = en;
                    self.current_ru = ru;
                    self.current_record_bytes = record_bytes;
                    self.phase = Phase::ShowPrompt;
                    Command::None
                }

                Core0Result::Error(msg) => {
                    log::error!("Review error: {}", msg);
                    self.phase = Phase::Error(msg);
                    Command::None
                }

                _ => Command::None,
            },

            _ => Command::None,
        }
    }

    fn snapshot(&self, shared: &SharedState) -> Snapshot {
        let pending_action = match &self.phase {
            Phase::Loading => Some(Core0Action::LoadDueItems),
            Phase::LoadingWord => {
                let item = &self.due_items[self.current_idx];
                Some(Core0Action::LoadWordText {
                    slot: item.slot,
                    word_offset: item.word_offset,
                    word_length: item.word_length,
                })
            }
            Phase::SavingRating => {
                let item = &self.due_items[self.current_idx];
                let next_idx = self.current_idx + 1;
                if next_idx < self.due_items.len() {
                    let next = &self.due_items[next_idx];
                    Some(Core0Action::RateAndLoadNext {
                        slot: item.slot,
                        record_bytes: self.current_record_bytes,
                        next_slot: next.slot,
                        next_word_offset: next.word_offset,
                        next_word_length: next.word_length,
                    })
                } else {
                    Some(Core0Action::RateCard {
                        slot: item.slot,
                        record_bytes: self.current_record_bytes,
                    })
                }
            }
            _ => None,
        };

        let lang = shared.lang;

        let phase_snap = match &self.phase {
            Phase::NtpCheck => PhaseSnapshot::NtpCheck,
            Phase::Loading | Phase::LoadingWord => PhaseSnapshot::Loading,
            Phase::ShowPrompt => {
                if let Some((prompt, _, dir)) = self.current_prompt_answer() {
                    PhaseSnapshot::ShowPrompt {
                        progress: format_progress(self.current_idx, self.total_count, dir),
                        prompt: prompt.to_string(),
                    }
                } else {
                    PhaseSnapshot::Error("Card not found".into())
                }
            }
            Phase::ShowAnswer => {
                if let Some((prompt, answer, dir)) = self.current_prompt_answer() {
                    PhaseSnapshot::ShowAnswer {
                        progress: format_progress(self.current_idx, self.total_count, dir),
                        prompt: prompt.to_string(),
                        answer: answer.to_string(),
                    }
                } else {
                    PhaseSnapshot::Error("Card not found".into())
                }
            }
            Phase::SavingRating => PhaseSnapshot::Saving,
            Phase::SessionDone => PhaseSnapshot::SessionDone { count: self.total_count },
            Phase::Empty => PhaseSnapshot::Empty,
            Phase::Error(msg) => PhaseSnapshot::Error(msg.clone()),
        };

        Snapshot::Review(ReviewSnapshot { phase: phase_snap, pending_action, lang })
    }
}

fn format_progress(idx: usize, total: usize, dir: Direction) -> String {
    let mut s = String::new();
    let _ = write!(s, "Card {} of {} [{}]", idx + 1, total, direction_label(dir));
    s
}

// ---------------------------------------------------------------------------
// Snapshot
// ---------------------------------------------------------------------------

#[derive(Clone)]
enum PhaseSnapshot {
    NtpCheck,
    Loading,
    ShowPrompt { progress: String, prompt: String },
    ShowAnswer { progress: String, prompt: String, answer: String },
    Saving,
    SessionDone { count: usize },
    Empty,
    Error(String),
}

pub struct ReviewSnapshot {
    phase: PhaseSnapshot,
    pub pending_action: Option<Core0Action>,
    lang: InputLanguage,
}

impl ReviewSnapshot {
    pub fn draw<FB: CardworderFB>(&self, ui: &mut CardworderUi<FB>) {
        let l = self.lang;
        let lines: Vec<UiLineType> = match &self.phase {
            PhaseSnapshot::ShowPrompt { progress, prompt } => vec![
                text_line(progress, CardFont::Small, ThemeColor::Text),
                UiLineType::Spacer(8),
                UiLineType::AutoText { text: prompt.clone(), color: ThemeColor::Selected, max_width: 232 },
                UiLineType::Spacer(8),
                text_line(t("Press Enter to reveal", "Нажмите Enter для ответа", l), CardFont::Small, ThemeColor::Text),
            ],
            PhaseSnapshot::ShowAnswer { progress, prompt, answer } => vec![
                text_line(progress, CardFont::Small, ThemeColor::Text),
                UiLineType::Spacer(4),
                UiLineType::AutoText { text: prompt.clone(), color: ThemeColor::Text, max_width: 232 },
                UiLineType::Spacer(4),
                UiLineType::AutoText { text: answer.clone(), color: ThemeColor::Color(Rgb565::new(0, 63, 0)), max_width: 232 },
                UiLineType::Spacer(4),
                text_line(t("1:Again 2:Hard 3:Good 4:Easy", "1:Снова 2:Трудно 3:Хорошо 4:Легко", l), CardFont::Medium, ThemeColor::Selected),
            ],
            PhaseSnapshot::NtpCheck => msg_lines(
                t("Time not set", "Время не установлено", l),
                t("Set time via NTP first", "Сначала синхронизируйте время", l),
                ThemeColor::Error, Some(t("Press Enter/Esc", "Нажмите Enter/Esc", l)),
            ),
            PhaseSnapshot::Loading => msg_lines(
                t("Review", "Повторение", l),
                t("Loading cards...", "Загрузка карточек...", l),
                ThemeColor::Text, None,
            ),
            PhaseSnapshot::Saving => msg_lines(
                t("Review", "Повторение", l),
                t("Saving progress...", "Сохранение прогресса...", l),
                ThemeColor::Text, None,
            ),
            PhaseSnapshot::SessionDone { count } => {
                let mut body = String::new();
                let _ = write!(body, "{}{}{}", t("Reviewed ", "Повторено ", l), count, t(" cards", " карточек", l));
                vec![
                    text_line(t("Done!", "Готово!", l), CardFont::Large, ThemeColor::Color(Rgb565::new(0, 63, 0))),
                    UiLineType::Spacer(6),
                    text_line_owned(&body, ThemeColor::Text),
                    UiLineType::Spacer(6),
                    text_line(t("Press Enter/Esc", "Нажмите Enter/Esc", l), CardFont::Small, ThemeColor::Text),
                ]
            }
            PhaseSnapshot::Empty => msg_lines(
                t("Review", "Повторение", l),
                t("No cards due!", "Нет карточек для повторения!", l),
                ThemeColor::Color(Rgb565::new(0, 63, 0)), Some(t("Press Enter/Esc", "Нажмите Enter/Esc", l)),
            ),
            PhaseSnapshot::Error(msg) => {
                vec![
                    text_line(t("Error", "Ошибка", l), CardFont::Large, ThemeColor::Error),
                    UiLineType::Spacer(6),
                    text_line_owned(msg, ThemeColor::Text),
                    UiLineType::Spacer(6),
                    text_line(t("Press Enter/Esc", "Нажмите Enter/Esc", l), CardFont::Small, ThemeColor::Text),
                ]
            }
        };

        let viewport_height = 131u32;
        let composed = compose_scrolled_form(&lines, 0, viewport_height, 4, ui);
        render_visible_lines(&composed, &lines, ui);
    }
}

fn text_line<'a>(text: &'a str, font: CardFont, color: ThemeColor) -> UiLineType<'a> {
    UiLineType::Elements(vec![UiLineElement::Text(text, font, u8g2_fonts::types::VerticalPosition::Top, color)])
}

fn text_line_owned<'a>(text: &str, color: ThemeColor) -> UiLineType<'a> {
    UiLineType::AutoText { text: text.to_string(), color, max_width: 232 }
}

fn msg_lines<'a>(title: &'a str, body: &'a str, body_color: ThemeColor, hint: Option<&'a str>) -> Vec<UiLineType<'a>> {
    let mut lines = vec![
        text_line(title, CardFont::Large, ThemeColor::Selected),
        UiLineType::Spacer(6),
        text_line(body, CardFont::Medium, body_color),
    ];
    if let Some(h) = hint {
        lines.push(UiLineType::Spacer(6));
        lines.push(text_line(h, CardFont::Small, ThemeColor::Text));
    }
    lines
}
