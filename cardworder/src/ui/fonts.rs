//! Font measurement helpers for Cardputer UI.

use crate::ui::cardworder_ui::CardFont;
use crate::ui::elements::{UiLineElement, UiLineType};

pub fn measure_element_height(ui: &crate::ui::cardworder_ui::CardworderUi, element: &UiLineElement) -> u32 {
    match element {
        UiLineElement::Icon(_, font, _) => ui.font_height(*font),
        UiLineElement::Text(_, font, _, _) => ui.font_height(*font),
        UiLineElement::Spacer(pixels) => *pixels as u32,
        UiLineElement::Filler => 0,
    }
}

pub fn measure_line_height(ui: &crate::ui::cardworder_ui::CardworderUi, line: &UiLineType) -> u32 {
    match line {
        UiLineType::Elements(elements) => {
            elements.iter().map(|e| measure_element_height(ui, e)).max().unwrap_or(0)
        }
        UiLineType::Spacer(pixels) => *pixels as u32,
        UiLineType::Line(pixels, _) => *pixels as u32,
        UiLineType::InputField { .. } => {
            ui.font_height(CardFont::Small) + 2 + ui.font_height(CardFont::Medium) + 6
        }
        UiLineType::AutoText { text, max_width, .. } => {
            measure_auto_text_height(ui, text, *max_width)
        }
    }
}

/// Measure height of auto-sized text (same logic as `draw_text_auto`).
pub fn measure_auto_text_height(ui: &crate::ui::cardworder_ui::CardworderUi, text: &str, max_width: u32) -> u32 {
    let char_count = text.chars().count();
    let xlarge_max = (max_width / 10) as usize;
    let large_max = (max_width / 9) as usize;
    let med_max = (max_width / 6) as usize;

    if char_count <= xlarge_max {
        ui.font_height(CardFont::XLarge)
    } else if char_count <= large_max {
        ui.font_height(CardFont::Large)
    } else if char_count <= med_max {
        ui.font_height(CardFont::Medium)
    } else {
        // Word wrap: count lines
        let line_h = ui.font_height(CardFont::Medium);
        let mut line_count = 1u32;
        let mut cur_len = 0usize;
        for word in text.split_whitespace() {
            let word_len = word.chars().count();
            if cur_len == 0 {
                cur_len = word_len;
            } else if cur_len + 1 + word_len <= med_max {
                cur_len += 1 + word_len;
            } else {
                line_count += 1;
                cur_len = word_len;
            }
        }
        line_count * line_h + (line_count - 1) // +1px gap between lines
    }
}
