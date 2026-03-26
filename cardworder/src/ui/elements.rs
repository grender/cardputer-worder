use u8g2_fonts::types::VerticalPosition;

use crate::ui::cardworder_ui::{CardFont, ThemeColor};

#[derive(Clone)]
/// A primitive UI element for a line in a form.
pub enum UiLineElement<'a> {
    Icon(char, CardFont, ThemeColor),
    Text(&'a str, CardFont, VerticalPosition, ThemeColor),
    Spacer(u8),
    Filler,
}

#[derive(Clone)]
/// A line in a form, which may be a set of elements, a spacer, a line, or an input field.
pub enum UiLineType<'a> {
    Elements(Vec<UiLineElement<'a>>),
    Spacer(u8),
    Line(u8, ThemeColor),
    /// Editable text input field with label, value, cursor, and focus state.
    InputField {
        label: &'a str,
        value: heapless::String<64>,
        cursor_pos: usize,
        focused: bool,
    },
}
