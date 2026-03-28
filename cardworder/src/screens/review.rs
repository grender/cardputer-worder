use core::fmt::Write;
use std::time::SystemTime;

use embedded_graphics::pixelcolor::Rgb565;
use embedded_graphics::prelude::Point;
use u8g2_fonts::types::VerticalPosition;

use crate::cardputer_hal::input::keyboard::{InputLanguage, PressedSymbol};
use crate::cardputer_hal::input::keyboard_io::KeyEvent;
use crate::screen::{Screen, Snapshot};
use crate::screens::main_menu::MainMenuScreen;
use crate::types::{Command, Core0Action, Core0Result, Msg, SharedState};
use crate::ui::cardworder_ui::{CardFont, CardworderUi, ThemeColor, TOP_BAR_HEIGHT};
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

    fn current_direction(&self) -> Option<Direction> {
        self.due_items.get(self.current_idx).map(|item| item.direction)
    }

    fn current_prompt_answer(&self) -> Option<(&str, &str, Direction)> {
        let item = self.due_items.get(self.current_idx)?;
        let (prompt, answer) = match item.direction {
            Direction::Forward => (self.current_en.as_str(), self.current_ru.as_str()),
            Direction::Reverse => (self.current_ru.as_str(), self.current_en.as_str()),
        };
        Some((prompt, answer, item.direction))
    }

    /// Apply rating to the current card's FSRS record in memory.
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
                    Some((KeyEvent::Pressed, PressedSymbol::Esc)) => {
                        Self::go_main_menu()
                    }
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
                    Some((KeyEvent::Pressed, PressedSymbol::Esc)) => {
                        Self::go_main_menu()
                    }
                    _ => Command::None,
                },

                Phase::SessionDone | Phase::Empty | Phase::Error(_) => match key_msg.pressed {
                    Some((KeyEvent::Pressed, PressedSymbol::Esc | PressedSymbol::Enter)) => Self::go_main_menu(),
                    _ => Command::None,
                },
            },

            Msg::Core0Result(result) => match result {
                Core0Result::DueItemsLoaded { forward_items, reverse_items, next_id: _ } => {
                    // Pick list based on interface language:
                    // English UI → EN→RU (forward), Russian UI → RU→EN (reverse)
                    // If preferred list is empty, fall back to the other
                    use crate::cardputer_hal::input::keyboard::InputLanguage;
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
                        let mut seed = unsafe { esp_idf_svc::sys::esp_timer_get_time() as u64 };
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
                    // Last card was rated (no next card)
                    self.current_idx += 1;
                    self.phase = Phase::SessionDone;
                    Command::None
                }

                Core0Result::CardRatedAndNextLoaded { en, ru, record_bytes } => {
                    // Current card saved + next card already loaded
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
                    // Combined: save current + load next in one SD operation
                    let next = &self.due_items[next_idx];
                    Some(Core0Action::RateAndLoadNext {
                        slot: item.slot,
                        record_bytes: self.current_record_bytes,
                        next_slot: next.slot,
                        next_word_offset: next.word_offset,
                        next_word_length: next.word_length,
                    })
                } else {
                    // Last card — just save
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
    pub fn draw(&self, ui: &mut CardworderUi) {
        let l = self.lang;
        match &self.phase {
            PhaseSnapshot::NtpCheck => self.draw_msg(ui,
                t("Time not set", "Время не установлено", l),
                t("Set time via NTP first", "Сначала синхронизируйте время", l),
                ThemeColor::Error, true),
            PhaseSnapshot::Loading => self.draw_msg(ui,
                t("Review", "Повторение", l),
                t("Loading cards...", "Загрузка карточек...", l),
                ThemeColor::Text, false),
            PhaseSnapshot::ShowPrompt { progress, prompt } => self.draw_prompt(ui, progress, prompt),
            PhaseSnapshot::ShowAnswer { progress, prompt, answer } => self.draw_answer(ui, progress, prompt, answer),
            PhaseSnapshot::Saving => self.draw_msg(ui,
                t("Review", "Повторение", l),
                t("Saving progress...", "Сохранение прогресса...", l),
                ThemeColor::Text, false),
            PhaseSnapshot::SessionDone { count } => {
                let mut body = String::new();
                let _ = write!(body, "{}{}{}", t("Reviewed ", "Повторено ", l), count, t(" cards", " карточек", l));
                self.draw_msg(ui, t("Done!", "Готово!", l), &body,
                    ThemeColor::Color(Rgb565::new(0, 63, 0)), true);
            }
            PhaseSnapshot::Empty => self.draw_msg(ui,
                t("Review", "Повторение", l),
                t("No cards due!", "Нет карточек для повторения!", l),
                ThemeColor::Color(Rgb565::new(0, 63, 0)), true),
            PhaseSnapshot::Error(msg) => self.draw_msg(ui,
                t("Error", "Ошибка", l), msg, ThemeColor::Error, true),
        }
    }

    fn draw_msg(&self, ui: &mut CardworderUi, title: &str, body: &str, color: ThemeColor, show_hint: bool) {
        let l = self.lang;
        let mut y = TOP_BAR_HEIGHT as i32 + 10;
        ui.draw_text_oneline(title, CardFont::Large, ThemeColor::Selected, Point::new(4, y), VerticalPosition::Top);
        y += ui.font_height(CardFont::Large) as i32 + 8;
        ui.draw_text_oneline(body, CardFont::Medium, color, Point::new(4, y), VerticalPosition::Top);
        if show_hint {
            y += ui.font_height(CardFont::Medium) as i32 + 8;
            ui.draw_text_oneline(t("Press Enter/Esc", "Нажмите Enter/Esc", l), CardFont::Small, ThemeColor::Text, Point::new(4, y), VerticalPosition::Top);
        }
    }

    fn draw_prompt(&self, ui: &mut CardworderUi, progress: &str, prompt: &str) {
        let l = self.lang;
        let mut y = TOP_BAR_HEIGHT as i32 + 6;
        ui.draw_text_oneline(progress, CardFont::Small, ThemeColor::Text, Point::new(4, y), VerticalPosition::Top);
        y += ui.font_height(CardFont::Small) as i32 + 12;
        let h = ui.draw_text_auto(prompt, ThemeColor::Selected, 4, y, 232);
        y += h + 12;
        ui.draw_text_oneline(t("Press Enter to reveal", "Нажмите Enter для ответа", l), CardFont::Small, ThemeColor::Text, Point::new(4, y), VerticalPosition::Top);
    }

    fn draw_answer(&self, ui: &mut CardworderUi, progress: &str, prompt: &str, answer: &str) {
        let l = self.lang;
        let mut y = TOP_BAR_HEIGHT as i32 + 6;
        ui.draw_text_oneline(progress, CardFont::Small, ThemeColor::Text, Point::new(4, y), VerticalPosition::Top);
        y += ui.font_height(CardFont::Small) as i32 + 6;
        let h = ui.draw_text_auto(prompt, ThemeColor::Text, 4, y, 232);
        y += h + 4;
        let h = ui.draw_text_auto(answer, ThemeColor::Color(Rgb565::new(0, 63, 0)), 4, y, 232);
        y += h + 6;
        ui.draw_text_oneline(
            t("1:Again 2:Hard 3:Good 4:Easy", "1:Снова 2:Трудно 3:Хорошо 4:Легко", l),
            CardFont::Medium, ThemeColor::Selected, Point::new(4, y), VerticalPosition::Top);
    }
}
