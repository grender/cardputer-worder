use u8g2_fonts::types::VerticalPosition;

use crate::ui::cardworder_ui::{CardFont, ThemeColor};

#[derive(Clone)]
pub enum UiLineElement<'a> {
    Icon(char, CardFont, ThemeColor),
    Text(&'a str, CardFont, VerticalPosition, ThemeColor),
    Spacer(u8),
    Filler,
}

#[derive(Clone)]
pub enum UiLineType<'a> {
    Elements(Vec<UiLineElement<'a>>),
    Spacer(u8),
    Line(u8, ThemeColor),
    InputField {
        label: &'a str,
        value: heapless::String<64>,
        cursor_pos: usize,
        focused: bool,
    },
    /// Auto-sized text: picks largest font that fits, wraps if needed.
    AutoText {
        text: String,
        color: ThemeColor,
        max_width: u32,
    },
}
