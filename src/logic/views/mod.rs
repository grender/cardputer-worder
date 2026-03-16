use u8g2_fonts::types::VerticalPosition;

use crate::ui::cardworder_ui::{CardFont, ThemeColor};

pub mod main_menu;
pub mod start;
pub mod fonts;
pub mod render;

#[derive(Clone)]
/// A primitive UI element for a line in a form.
pub enum UiLineElement {
    Icon(char, CardFont, ThemeColor),
    Text(&'static str, CardFont, VerticalPosition, ThemeColor),
    Spacer(u8),
    Filler,
}

#[derive(Clone)]
/// A line in a form, which may be a set of elements, a spacer, or a line.
pub enum UiLineType {
    Elements(Vec<UiLineElement>),
    Spacer(u8),
    Line(u8, ThemeColor),
}
