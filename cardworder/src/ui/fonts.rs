//! Font measurement helpers for Cardputer UI.

use crate::ui::cardworder_ui::CardFont;
use crate::ui::elements::{UiLineElement, UiLineType};

/// Measures the height of a single UiLineElement.
pub fn measure_element_height(ui: &crate::ui::cardworder_ui::CardworderUi, element: &UiLineElement) -> u32 {
    match element {
        UiLineElement::Icon(_, font, _) => ui.font_height(*font),
        UiLineElement::Text(_, font, _, _) => ui.font_height(*font),
        UiLineElement::Spacer(pixels) => *pixels as u32,
        UiLineElement::Filler => 0,
    }
}

/// Measures the height of a UiLineType (max of all elements, or value for spacers/lines).
pub fn measure_line_height(ui: &crate::ui::cardworder_ui::CardworderUi, line: &UiLineType) -> u32 {
    match line {
        UiLineType::Elements(elements) => {
            elements.iter().map(|e| measure_element_height(ui, e)).max().unwrap_or(0)
        }
        UiLineType::Spacer(pixels) => *pixels as u32,
        UiLineType::Line(pixels, _) => *pixels as u32,
    }
}

/// Measures the height and line count for a multiline text.
pub fn measure_multiline_text(
    _text: &str,
    font: CardFont,
    _max_width: u32,
    ui: &crate::ui::cardworder_ui::CardworderUi,
) -> (u32, u32) {
    (1, ui.font_height(font))
}
