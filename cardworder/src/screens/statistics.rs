use core::fmt::Write;
use u8g2_fonts::types::VerticalPosition;

use crate::cardputer_hal::cardputer_hal::{QuickStats, DirStats};
use crate::cardputer_hal::input::keyboard::{InputLanguage, PressedSymbol};
use crate::cardputer_hal::input::keyboard_io::KeyEvent;
use crate::screen::{Screen, Snapshot};
use crate::screens::main_menu::MainMenuScreen;
use crate::types::{Command, Core0Action, Core0Result, Msg, SharedState};
use crate::ui::cardworder_ui::{CardFont, CardworderUi, ThemeColor, TOP_BAR_HEIGHT};
use crate::ui::elements::{UiLineElement, UiLineType};
use crate::ui::render::{compose_scrolled_form, render_visible_lines};
use embedded_graphics::pixelcolor::Rgb565;

fn t(en: &'static str, ru: &'static str, lang: InputLanguage) -> &'static str {
    match lang { InputLanguage::En => en, InputLanguage::Ru => ru }
}

enum Phase { Loading, Loaded, Error(String) }

pub struct StatisticsScreen {
    phase: Phase,
    stats: Option<QuickStats>,
    focused_idx: usize,
    active_tab: usize, // 0 = EN→RU, 1 = RU→EN
}

impl StatisticsScreen {
    pub fn new() -> Self {
        Self { phase: Phase::Loading, stats: None, focused_idx: 0, active_tab: 0 }
    }
}

impl Screen for StatisticsScreen {
    fn on_mount(&mut self, _shared: &SharedState, _state_tx: &std::sync::mpsc::Sender<Snapshot>) -> Command {
        self.phase = Phase::Loading;
        self.stats = None;
        self.focused_idx = 0;
        Command::None
    }

    fn handle_msg(&mut self, msg: Msg, _shared: &SharedState) -> Command {
        match msg {
            Msg::Core0Result(result) => {
                match result {
                    Core0Result::QuickStatsLoaded(stats) => {
                        self.stats = Some(stats);
                        self.phase = Phase::Loaded;
                    }
                    Core0Result::Error(e) => { self.phase = Phase::Error(e); }
                    _ => {}
                }
                Command::None
            }
            Msg::Key(key_msg) => match key_msg.pressed {
                Some((KeyEvent::Pressed, PressedSymbol::Esc | PressedSymbol::Enter)) => {
                    Command::SwitchTo(Box::new(MainMenuScreen::default()))
                }
                Some((KeyEvent::Pressed, PressedSymbol::ArrowLeft)) => {
                    if self.active_tab > 0 { self.active_tab -= 1; self.focused_idx = 0; }
                    Command::None
                }
                Some((KeyEvent::Pressed, PressedSymbol::ArrowRight)) => {
                    if self.active_tab < 1 { self.active_tab += 1; self.focused_idx = 0; }
                    Command::None
                }
                Some((KeyEvent::Pressed, PressedSymbol::ArrowDown)) => {
                    self.focused_idx += 1;
                    Command::None
                }
                Some((KeyEvent::Pressed, PressedSymbol::ArrowUp)) => {
                    self.focused_idx = self.focused_idx.saturating_sub(1);
                    Command::None
                }
                _ => Command::None,
            },
            _ => Command::None,
        }
    }

    fn snapshot(&self, shared: &SharedState) -> Snapshot {
        let pending_action = match &self.phase {
            Phase::Loading if self.stats.is_none() => Some(Core0Action::LoadQuickStats),
            _ => None,
        };
        Snapshot::Statistics(StatisticsSnapshot {
            stats: self.stats.clone(),
            focused_idx: self.focused_idx,
            active_tab: self.active_tab,
            loading: matches!(self.phase, Phase::Loading),
            error: if let Phase::Error(e) = &self.phase { Some(e.clone()) } else { None },
            lang: shared.lang,
            pending_action,
        })
    }
}

// ---- Snapshot ----

pub struct StatisticsSnapshot {
    stats: Option<QuickStats>,
    focused_idx: usize,
    active_tab: usize,
    loading: bool,
    error: Option<String>,
    lang: InputLanguage,
    pub pending_action: Option<Core0Action>,
}

impl StatisticsSnapshot {
    pub fn draw(&self, ui: &mut CardworderUi) {
        let l = self.lang;
        let med = CardFont::Medium;
        let sm = CardFont::Small;
        let text = ThemeColor::Text;
        let dim = ThemeColor::Color(Rgb565::new(16, 32, 16));
        let green = ThemeColor::Color(Rgb565::new(0, 50, 0));
        let yellow = ThemeColor::Color(Rgb565::new(31, 50, 0));
        let red = ThemeColor::Error;

        const SCREEN_H: u32 = 135;
        let vh = SCREEN_H - TOP_BAR_HEIGHT;

        if self.loading {
            let lines = vec![UiLineType::Elements(vec![
                UiLineElement::Text(t("Loading...", "Загрузка...", l), med, VerticalPosition::Top, text),
            ])];
            let c = compose_scrolled_form(&lines, 0, vh, TOP_BAR_HEIGHT as i32, ui);
            render_visible_lines(&c, &lines, ui);
            return;
        }

        if let Some(err) = &self.error {
            let lines = vec![
                UiLineType::Elements(vec![UiLineElement::Text(t("Error", "Ошибка", l), CardFont::Large, VerticalPosition::Top, red)]),
                UiLineType::Elements(vec![UiLineElement::Text(err.as_str(), sm, VerticalPosition::Top, text)]),
                UiLineType::Spacer(4),
                UiLineType::Elements(vec![UiLineElement::Text(t("<- Back", "<- Назад", l), med, VerticalPosition::Top, ThemeColor::Selected)]),
            ];
            let c = compose_scrolled_form(&lines, 0, vh, TOP_BAR_HEIGHT as i32, ui);
            render_visible_lines(&c, &lines, ui);
            return;
        }

        let s = match &self.stats { Some(s) => s, None => return };

        let ds = if self.active_tab == 0 { &s.forward } else { &s.reverse };

        // Tab header
        let tab0_color = if self.active_tab == 0 { ThemeColor::Selected } else { dim };
        let tab1_color = if self.active_tab == 1 { ThemeColor::Selected } else { dim };

        macro_rules! fmt {
            ($($arg:tt)*) => {{ let mut s = heapless::String::<64>::new(); let _ = write!(s, $($arg)*); s }};
        }

        let tab_label = fmt!("{}: {}", t("Total pairs", "Всего пар", l), s.total_pairs);
        let b_due = fmt!("{}: {}", t("Due", "К повтору", l), ds.due);
        let b_new = fmt!("{}: {}", t("New", "Новые", l), ds.new_count);
        let b_learn = fmt!("{}: {}", t("Learning", "Изучаемые", l), ds.learning);
        let b_mast = fmt!("{}: {}", t("Mastered", "Выученные", l), ds.mastered);
        let b_rev = fmt!("{}: {}", t("Reviews", "Повторений", l), ds.total_reviews);
        let b_lap = fmt!("{}: {}", t("Lapses", "Забываний", l), ds.total_lapses);
        let b_diff = fmt!("{}: {:.1}", t("Avg difficulty", "Ср. сложность", l), ds.avg_difficulty);

        let mut lines: Vec<UiLineType> = vec![
            // Tab header
            UiLineType::Elements(vec![
                UiLineElement::Text(if self.active_tab == 0 { "[EN>RU]" } else { " EN>RU " }, med, VerticalPosition::Top, tab0_color),
                UiLineElement::Spacer(8),
                UiLineElement::Text(if self.active_tab == 1 { "[RU>EN]" } else { " RU>EN " }, med, VerticalPosition::Top, tab1_color),
            ]),
            UiLineType::Line(1, dim),
            UiLineType::Elements(vec![UiLineElement::Text(tab_label.as_str(), sm, VerticalPosition::Top, text)]),
            UiLineType::Elements(vec![UiLineElement::Text(b_due.as_str(), sm, VerticalPosition::Top, if ds.due > 0 { yellow } else { dim })]),
            UiLineType::Line(1, dim),
            UiLineType::Elements(vec![UiLineElement::Text(b_new.as_str(), sm, VerticalPosition::Top, text)]),
            UiLineType::Elements(vec![UiLineElement::Text(b_learn.as_str(), sm, VerticalPosition::Top, yellow)]),
            UiLineType::Elements(vec![UiLineElement::Text(b_mast.as_str(), sm, VerticalPosition::Top, green)]),
            UiLineType::Line(1, dim),
            UiLineType::Elements(vec![UiLineElement::Text(b_rev.as_str(), sm, VerticalPosition::Top, text)]),
            UiLineType::Elements(vec![UiLineElement::Text(b_lap.as_str(), sm, VerticalPosition::Top, if ds.total_lapses > 0 { red } else { dim })]),
            UiLineType::Elements(vec![UiLineElement::Text(b_diff.as_str(), sm, VerticalPosition::Top, text)]),
            UiLineType::Line(1, dim),
        ];

        let mut h_hard = heapless::String::<80>::new();
        let mut h_strong = heapless::String::<80>::new();
        let mut h_weak = heapless::String::<80>::new();

        if let Some(ref w) = ds.hardest_word {
            let _ = write!(h_hard, "{}: {} ({}x)", t("Hardest", "Сложное", l), w, ds.hardest_lapses);
            lines.push(UiLineType::Elements(vec![UiLineElement::Text(h_hard.as_str(), sm, VerticalPosition::Top, red)]));
        }
        if let Some(ref w) = ds.strongest_word {
            let _ = write!(h_strong, "{}: {} ({}d)", t("Strongest", "Лучшее", l), w, ds.strongest_days);
            lines.push(UiLineType::Elements(vec![UiLineElement::Text(h_strong.as_str(), sm, VerticalPosition::Top, green)]));
        }
        if let Some(ref w) = ds.weakest_word {
            let _ = write!(h_weak, "{}: {} ({}d)", t("Weakest", "Слабое", l), w, ds.weakest_days);
            lines.push(UiLineType::Elements(vec![UiLineElement::Text(h_weak.as_str(), sm, VerticalPosition::Top, yellow)]));
        }

        lines.push(UiLineType::Spacer(4));
        lines.push(UiLineType::Elements(vec![UiLineElement::Text(
            t("<- Back (Enter/Esc)", "<- Назад (Enter/Esc)", l), med, VerticalPosition::Top, ThemeColor::Selected,
        )]));

        let c = compose_scrolled_form(&lines, self.focused_idx.min(lines.len().saturating_sub(1)), vh, TOP_BAR_HEIGHT as i32, ui);
        render_visible_lines(&c, &lines, ui);
    }
}
