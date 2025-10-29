use core::iter;
use embedded_graphics::{pixelcolor::Rgb565, prelude::Size, primitives::Rectangle};
use u8g2_fonts::types::VerticalPosition;

use crate::ui::cardworder_ui::{CardFont, ThemeColor};

pub mod main_menu;
pub mod start;
pub mod fonts;
pub mod render;

#[derive(Clone)]
/// A primitive UI element for a line in a form.
pub enum UiLineElement<'a> {
    Icon(char, CardFont, ThemeColor),
    Text(&'static str, CardFont, VerticalPosition, ThemeColor),
    InputText(&'a String,CardFont, ThemeColor, u32), // input value, font, color, cursor_position
    Spacer(u8),
    Filler,
}

#[derive(Clone)]
/// A line in a form, which may be a set of elements, a spacer, or a line.
pub enum UiLineType<'a> {
    Elements(Vec<UiLineElement<'a>>),
    Spacer(u8),
    Line(u8, ThemeColor),
}
