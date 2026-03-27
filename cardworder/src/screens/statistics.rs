use core::fmt::Write;

use crate::cardputer_hal::input::keyboard::{InputLanguage, PressedSymbol};
use crate::cardputer_hal::input::keyboard_io::KeyEvent;
use crate::screen::{Screen, Snapshot};
use crate::screens::main_menu::MainMenuScreen;
use crate::types::{Command, Core0Action, Core0Result, Msg, SharedState};
use crate::ui::cardworder_ui::{CardFont, CardworderUi, ThemeColor, TOP_BAR_HEIGHT};
use crate::ui::elements::{UiLineElement, UiLineType};
use crate::ui::render::{compose_scrolled_form, render_visible_lines};
use fsrs_core::{Direction, PairsFile, is_card_due};
use u8g2_fonts::types::VerticalPosition;

use embedded_graphics::pixelcolor::Rgb565;
use embedded_graphics::prelude::WebColors;

fn t(en: &'static str, ru: &'static str, lang: InputLanguage) -> &'static str {
    match lang {
        InputLanguage::En => en,
        InputLanguage::Ru => ru,
    }
}

// ---------------------------------------------------------------------------
// Phase
// ---------------------------------------------------------------------------

enum Phase {
    Loading,
    Loaded,
    Error(String),
}

// ---------------------------------------------------------------------------
// Per-pair detail row (pre-computed to avoid holding the full PairsFile)
// ---------------------------------------------------------------------------

struct PairStatRow {
    label: String,
    fwd_info: String,
    rev_info: String,
    fwd_due: bool,
    rev_due: bool,
}

// ---------------------------------------------------------------------------
// Screen
// ---------------------------------------------------------------------------

pub struct StatisticsScreen {
    phase: Phase,
    focused_idx: usize,
    total_pairs: usize,
    due_fwd: usize,
    due_rev: usize,
    rows: Vec<PairStatRow>,
    needs_load: bool,
}

impl StatisticsScreen {
    pub fn new() -> Self {
        Self {
            phase: Phase::Loading,
            focused_idx: 0,
            total_pairs: 0,
            due_fwd: 0,
            due_rev: 0,
            rows: Vec::new(),
            needs_load: true,
        }
    }

    fn process_pairs(&mut self, file: &PairsFile) {
        self.total_pairs = file.pairs.len();
        self.due_fwd = 0;
        self.due_rev = 0;
        self.rows.clear();

        for pair in &file.pairs {
            let fwd_state = pair.direction_state(Direction::Forward);
            let rev_state = pair.direction_state(Direction::Reverse);

            let fwd_due = is_card_due(&fwd_state.card);
            let rev_due = is_card_due(&rev_state.card);

            if fwd_due {
                self.due_fwd += 1;
            }
            if rev_due {
                self.due_rev += 1;
            }

            let mut label = String::new();
            let _ = write!(label, "{} - {}", pair.en, pair.ru);

            let mut fwd_info = String::new();
            let _ = write!(fwd_info, "F:{}r", fwd_state.card.reps);

            let mut rev_info = String::new();
            let _ = write!(rev_info, "R:{}r", rev_state.card.reps);

            self.rows.push(PairStatRow {
                label,
                fwd_info,
                rev_info,
                fwd_due,
                rev_due,
            });
        }

        self.phase = Phase::Loaded;
    }
}

impl Screen for StatisticsScreen {
    fn on_mount(
        &mut self,
        _shared: &SharedState,
        _state_tx: &std::sync::mpsc::Sender<Snapshot>,
    ) -> Command {
        self.phase = Phase::Loading;
        self.needs_load = true;
        Command::None
    }

    fn handle_msg(&mut self, msg: Msg, _shared: &SharedState) -> Command {
        match msg {
            Msg::Core0Result(result) => {
                match result {
                    Core0Result::PairsLoaded(file) => {
                        self.needs_load = false;
                        self.process_pairs(&file);
                    }
                    Core0Result::Error(e) => {
                        self.needs_load = false;
                        self.phase = Phase::Error(e);
                    }
                    _ => {}
                }
                Command::None
            }
            Msg::Key(key_msg) => match key_msg.pressed {
                Some((KeyEvent::Pressed, PressedSymbol::Esc)) => {
                    Command::SwitchTo(Box::new(MainMenuScreen::default()))
                }
                Some((KeyEvent::Pressed, PressedSymbol::Enter)) => {
                    Command::SwitchTo(Box::new(MainMenuScreen::default()))
                }
                Some((KeyEvent::Pressed, PressedSymbol::ArrowDown)) => {
                    self.focused_idx += 1;
                    Command::None
                }
                Some((KeyEvent::Pressed, PressedSymbol::ArrowUp)) => {
                    if self.focused_idx > 0 {
                        self.focused_idx -= 1;
                    }
                    Command::None
                }
                _ => Command::None,
            },
            _ => Command::None,
        }
    }

    fn snapshot(&self, shared: &SharedState) -> Snapshot {
        let pending_action = if self.needs_load {
            Some(Core0Action::LoadPairs)
        } else {
            None
        };

        Snapshot::Statistics(StatisticsSnapshot {
            phase: match &self.phase {
                Phase::Loading => SnapshotPhase::Loading,
                Phase::Loaded => SnapshotPhase::Loaded,
                Phase::Error(e) => SnapshotPhase::Error(e.clone()),
            },
            focused_idx: self.focused_idx,
            total_pairs: self.total_pairs,
            due_fwd: self.due_fwd,
            due_rev: self.due_rev,
            rows: self
                .rows
                .iter()
                .map(|r| SnapshotPairRow {
                    label: r.label.clone(),
                    fwd_info: r.fwd_info.clone(),
                    rev_info: r.rev_info.clone(),
                    fwd_due: r.fwd_due,
                    rev_due: r.rev_due,
                })
                .collect(),
            lang: shared.lang,
            pending_action,
        })
    }
}

// ---------------------------------------------------------------------------
// Snapshot (sent to Core 0 for drawing)
// ---------------------------------------------------------------------------

enum SnapshotPhase {
    Loading,
    Loaded,
    Error(String),
}

struct SnapshotPairRow {
    label: String,
    fwd_info: String,
    rev_info: String,
    fwd_due: bool,
    rev_due: bool,
}

pub struct StatisticsSnapshot {
    phase: SnapshotPhase,
    focused_idx: usize,
    total_pairs: usize,
    due_fwd: usize,
    due_rev: usize,
    rows: Vec<SnapshotPairRow>,
    lang: InputLanguage,
    pub pending_action: Option<Core0Action>,
}

impl StatisticsSnapshot {
    pub fn draw(&self, ui: &mut CardworderUi) {
        let font = CardFont::Medium;
        let color = ThemeColor::Text;
        let label_color = ThemeColor::Color(Rgb565::CSS_GRAY);
        let l = self.lang;

        const SCREEN_HEIGHT: u32 = 135;
        let viewport_height = SCREEN_HEIGHT - TOP_BAR_HEIGHT;

        match &self.phase {
            SnapshotPhase::Loading => {
                let lines: Vec<UiLineType> = vec![UiLineType::Elements(vec![
                    UiLineElement::Text(
                        t("Loading statistics...", "Загрузка статистики...", l),
                        font,
                        VerticalPosition::Top,
                        color,
                    ),
                ])];
                let composed = compose_scrolled_form(&lines, 0, viewport_height, TOP_BAR_HEIGHT as i32, ui);
                render_visible_lines(&composed, &lines, ui);
            }
            SnapshotPhase::Error(err) => {
                let lines: Vec<UiLineType> = vec![
                    UiLineType::Elements(vec![UiLineElement::Text(
                        t("Error", "Ошибка", l),
                        CardFont::Large,
                        VerticalPosition::Top,
                        ThemeColor::Selected,
                    )]),
                    UiLineType::Elements(vec![UiLineElement::Text(
                        err.as_str(),
                        font,
                        VerticalPosition::Top,
                        color,
                    )]),
                    UiLineType::Spacer(4),
                    UiLineType::Elements(vec![UiLineElement::Text(
                        t("<- Back (Enter/Esc)", "<- Назад (Enter/Esc)", l),
                        font,
                        VerticalPosition::Top,
                        ThemeColor::Selected,
                    )]),
                ];
                let composed = compose_scrolled_form(
                    &lines,
                    self.focused_idx.min(lines.len().saturating_sub(1)),
                    viewport_height,
                    TOP_BAR_HEIGHT as i32,
                    ui,
                );
                render_visible_lines(&composed, &lines, ui);
            }
            SnapshotPhase::Loaded => {
                let due_total = self.due_fwd + self.due_rev;

                let mut s_total = heapless::String::<64>::new();
                let _ = write!(
                    s_total,
                    "{}: {}",
                    t("Total pairs", "Всего пар", l),
                    self.total_pairs
                );

                let mut s_due_fwd = heapless::String::<64>::new();
                let _ = write!(
                    s_due_fwd,
                    "{}: {}",
                    t("Due EN>RU", "К повтору EN>RU", l),
                    self.due_fwd
                );

                let mut s_due_rev = heapless::String::<64>::new();
                let _ = write!(
                    s_due_rev,
                    "{}: {}",
                    t("Due RU>EN", "К повтору RU>EN", l),
                    self.due_rev
                );

                let mut s_due_total = heapless::String::<64>::new();
                let _ = write!(
                    s_due_total,
                    "{}: {}",
                    t("Due total", "К повтору всего", l),
                    due_total
                );

                let mut lines: Vec<UiLineType> = vec![
                    UiLineType::Elements(vec![UiLineElement::Text(
                        t("Statistics", "Статистика", l),
                        CardFont::Large,
                        VerticalPosition::Top,
                        ThemeColor::Selected,
                    )]),
                    UiLineType::Elements(vec![UiLineElement::Text(
                        s_total.as_str(),
                        font,
                        VerticalPosition::Top,
                        color,
                    )]),
                    UiLineType::Elements(vec![UiLineElement::Text(
                        s_due_fwd.as_str(),
                        font,
                        VerticalPosition::Top,
                        color,
                    )]),
                    UiLineType::Elements(vec![UiLineElement::Text(
                        s_due_rev.as_str(),
                        font,
                        VerticalPosition::Top,
                        color,
                    )]),
                    UiLineType::Elements(vec![UiLineElement::Text(
                        s_due_total.as_str(),
                        font,
                        VerticalPosition::Top,
                        label_color,
                    )]),
                    UiLineType::Spacer(4),
                ];

                // Per-pair detail rows
                let mut pair_strings: Vec<heapless::String<128>> = Vec::with_capacity(self.rows.len());
                for row in &self.rows {
                    let mut s = heapless::String::<128>::new();
                    let _ = write!(s, "{} [{}  {}]", row.label, row.fwd_info, row.rev_info);
                    pair_strings.push(s);
                }

                for (i, (row, s)) in self.rows.iter().zip(pair_strings.iter()).enumerate() {
                    let row_color = if row.fwd_due || row.rev_due {
                        ThemeColor::Color(Rgb565::new(31, 40, 0)) // yellowish for due
                    } else {
                        label_color
                    };
                    let _ = i; // index available if needed
                    lines.push(UiLineType::Elements(vec![UiLineElement::Text(
                        s.as_str(),
                        font,
                        VerticalPosition::Top,
                        row_color,
                    )]));
                }

                lines.push(UiLineType::Spacer(4));
                lines.push(UiLineType::Elements(vec![UiLineElement::Text(
                    t("<- Back (Enter/Esc)", "<- Назад (Enter/Esc)", l),
                    font,
                    VerticalPosition::Top,
                    ThemeColor::Selected,
                )]));

                let composed = compose_scrolled_form(
                    &lines,
                    self.focused_idx.min(lines.len().saturating_sub(1)),
                    viewport_height,
                    TOP_BAR_HEIGHT as i32,
                    ui,
                );
                render_visible_lines(&composed, &lines, ui);
            }
        }
    }
}
