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
use fsrs_core::{Direction, PairsFile, Rating, WordPairId, is_card_due};
use fsrs_core::models::get_timestamp_iso;

#[derive(Clone)]
enum Phase {
    NtpCheck,
    Loading,
    ShowPrompt,
    ShowAnswer,
    Saving,       // batch save at end or on Esc
    SessionDone,
    Empty,
    Error(String),
}

struct DueItem {
    pair_id: WordPairId,
    direction: Direction,
}

pub struct ReviewScreen {
    phase: Phase,
    pairs_file: Option<PairsFile>,
    due_items: Vec<DueItem>,
    current_idx: usize,
    total_reviewed: usize,
    dirty: bool, // any ratings applied since last save?
    save_bytes: Option<Vec<u8>>,
}

impl ReviewScreen {
    pub fn new() -> Self {
        Self {
            phase: Phase::NtpCheck,
            pairs_file: None,
            due_items: Vec::new(),
            current_idx: 0,
            total_reviewed: 0,
            dirty: false,
            save_bytes: None,
        }
    }

    fn go_main_menu() -> Command {
        Command::SwitchTo(Box::new(MainMenuScreen::default()))
    }

    fn current_prompt_answer(&self) -> Option<(&str, &str, Direction)> {
        let item = self.due_items.get(self.current_idx)?;
        let file = self.pairs_file.as_ref()?;
        let pair = file.pairs.iter().find(|p| p.id == item.pair_id)?;
        let (prompt, answer) = match item.direction {
            Direction::Forward => (pair.en.as_str(), pair.ru.as_str()),
            Direction::Reverse => (pair.ru.as_str(), pair.en.as_str()),
        };
        Some((prompt, answer, item.direction))
    }

    /// Apply rating to current card in memory (no save yet)
    fn apply_rating(&mut self, rating: Rating) {
        let item = &self.due_items[self.current_idx];
        let pair_id = item.pair_id;
        let direction = item.direction;

        if let Some(ref mut file) = self.pairs_file {
            if let Some(pair) = file.pairs.iter_mut().find(|p| p.id == pair_id) {
                let state = pair.direction_state_mut(direction);
                let _ = fsrs_core::fsrs::update_card_with_review(&mut state.card, rating);
                state.last_review = Some(get_timestamp_iso());
            }
            self.dirty = true;
        }
    }

    /// Start batch save — serialize and enter Saving phase
    fn start_save(&mut self) {
        if !self.dirty {
            return;
        }
        if let Some(ref file) = self.pairs_file {
            match postcard::to_allocvec(file) {
                Ok(bytes) => {
                    self.save_bytes = Some(bytes);
                    self.phase = Phase::Saving;
                }
                Err(e) => {
                    self.phase = Phase::Error(format!("Serialize: {:?}", e));
                }
            }
        }
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

    fn handle_msg(&mut self, msg: Msg, _shared: &SharedState) -> Command {
        match msg {
            Msg::Key(key_msg) => match &self.phase {
                Phase::NtpCheck => match key_msg.pressed {
                    Some((KeyEvent::Pressed, PressedSymbol::Esc | PressedSymbol::Enter)) => Self::go_main_menu(),
                    _ => Command::None,
                },

                Phase::Loading | Phase::Saving => Command::None,

                Phase::ShowPrompt => match key_msg.pressed {
                    Some((KeyEvent::Pressed, PressedSymbol::Enter)) => {
                        self.phase = Phase::ShowAnswer;
                        Command::None
                    }
                    Some((KeyEvent::Pressed, PressedSymbol::Esc)) => {
                        // Save before leaving if dirty
                        if self.dirty {
                            self.start_save();
                            Command::None // will go to menu after save completes
                        } else {
                            Self::go_main_menu()
                        }
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
                        self.current_idx += 1;
                        if self.current_idx >= self.due_items.len() {
                            // Session done — save now
                            self.start_save();
                        } else {
                            self.phase = Phase::ShowPrompt;
                        }
                        Command::None
                    }
                    Some((KeyEvent::Pressed, PressedSymbol::Esc)) => {
                        if self.dirty {
                            self.start_save();
                            Command::None
                        } else {
                            Self::go_main_menu()
                        }
                    }
                    _ => Command::None,
                },

                Phase::SessionDone | Phase::Empty | Phase::Error(_) => match key_msg.pressed {
                    Some((KeyEvent::Pressed, PressedSymbol::Esc | PressedSymbol::Enter)) => Self::go_main_menu(),
                    _ => Command::None,
                },
            },

            Msg::Core0Result(result) => match result {
                Core0Result::PairsLoaded(file) => {
                    let mut due: Vec<DueItem> = Vec::new();
                    for pair in &file.pairs {
                        for dir in [Direction::Forward, Direction::Reverse] {
                            if is_card_due(&pair.direction_state(dir).card) {
                                due.push(DueItem { pair_id: pair.id, direction: dir });
                            }
                        }
                    }
                    self.pairs_file = Some(file);

                    if due.is_empty() {
                        self.phase = Phase::Empty;
                    } else {
                        let mut seed = unsafe { esp_idf_svc::sys::esp_timer_get_time() as u64 };
                        for i in (1..due.len()).rev() {
                            seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
                            let j = (seed >> 33) as usize % (i + 1);
                            due.swap(i, j);
                        }
                        self.total_reviewed = due.len();
                        self.due_items = due;
                        self.current_idx = 0;
                        self.phase = Phase::ShowPrompt;
                    }
                    Command::None
                }

                Core0Result::PairsSaved => {
                    self.save_bytes = None;
                    self.dirty = false;
                    // After save, go to SessionDone or MainMenu
                    if self.current_idx >= self.due_items.len() {
                        self.phase = Phase::SessionDone;
                    } else {
                        // Was saving due to Esc mid-session
                        return Self::go_main_menu();
                    }
                    Command::None
                }

                Core0Result::Error(msg) => {
                    log::error!("Review error: {}", msg);
                    self.save_bytes = None;
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
            Phase::Loading => Some(Core0Action::LoadPairs),
            Phase::Saving => self.save_bytes.as_ref().map(|b| Core0Action::SavePairsBytes(b.clone())),
            _ => None,
        };

        let lang = shared.lang;

        let phase_snap = match &self.phase {
            Phase::NtpCheck => PhaseSnapshot::NtpCheck,
            Phase::Loading => PhaseSnapshot::Loading,
            Phase::ShowPrompt => {
                if let Some((prompt, _, dir)) = self.current_prompt_answer() {
                    PhaseSnapshot::ShowPrompt {
                        progress: format_progress(self.current_idx, self.due_items.len(), dir),
                        prompt: prompt.to_string(),
                    }
                } else {
                    PhaseSnapshot::Error("Card not found".into())
                }
            }
            Phase::ShowAnswer => {
                if let Some((prompt, answer, dir)) = self.current_prompt_answer() {
                    PhaseSnapshot::ShowAnswer {
                        progress: format_progress(self.current_idx, self.due_items.len(), dir),
                        prompt: prompt.to_string(),
                        answer: answer.to_string(),
                    }
                } else {
                    PhaseSnapshot::Error("Card not found".into())
                }
            }
            Phase::Saving => PhaseSnapshot::Saving,
            Phase::SessionDone => PhaseSnapshot::SessionDone { count: self.total_reviewed },
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
        y += ui.font_height(CardFont::Small) as i32 + 16;
        ui.draw_text_oneline(prompt, CardFont::XLarge, ThemeColor::Selected, Point::new(4, y), VerticalPosition::Top);
        y += ui.font_height(CardFont::XLarge) as i32 + 16;
        ui.draw_text_oneline(t("Press Enter to reveal", "Нажмите Enter для ответа", l), CardFont::Small, ThemeColor::Text, Point::new(4, y), VerticalPosition::Top);
    }

    fn draw_answer(&self, ui: &mut CardworderUi, progress: &str, prompt: &str, answer: &str) {
        let l = self.lang;
        let mut y = TOP_BAR_HEIGHT as i32 + 6;
        ui.draw_text_oneline(progress, CardFont::Small, ThemeColor::Text, Point::new(4, y), VerticalPosition::Top);
        y += ui.font_height(CardFont::Small) as i32 + 8;
        ui.draw_text_oneline(prompt, CardFont::Medium, ThemeColor::Text, Point::new(4, y), VerticalPosition::Top);
        y += ui.font_height(CardFont::Medium) as i32 + 4;
        ui.draw_text_oneline(answer, CardFont::XLarge, ThemeColor::Color(Rgb565::new(0, 63, 0)), Point::new(4, y), VerticalPosition::Top);
        y += ui.font_height(CardFont::XLarge) as i32 + 8;
        ui.draw_text_oneline(
            t("1:Again 2:Hard 3:Good 4:Easy", "1:Снова 2:Трудно 3:Хорошо 4:Легко", l),
            CardFont::Medium, ThemeColor::Selected, Point::new(4, y), VerticalPosition::Top);
    }
}
