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
    }
}
