//! Compose, scroll, and render logic for scrollable forms in Cardputer UI.

use crate::logic::views::{UiLineType, UiLineElement};
use crate::ui::cardworder_ui::CardworderUi;

/// State for a composed form, including scroll offset.
pub struct ComposedForm {
    pub lines: Vec<ComposedUiLine>,
    pub scroll_offset: u32,
    pub viewport_height: u32,
}

/// A composed line with its rectangle.
pub struct ComposedUiLine {
    pub line: UiLineType,
    pub rect: embedded_graphics::primitives::Rectangle,
}

/// Compose the form: calculate line heights, assign rectangles, and return visible lines.
pub fn compose_form(
    lines: &[UiLineType],
    scroll_offset: u32,
    viewport_height: u32,
    ui: &crate::ui::cardworder_ui::CardworderUi,
) -> ComposedForm {
    let mut composed_lines = Vec::new();
    let mut y = 0u32;
    let width = 240; // Assume fixed width for now
    let viewport_top = scroll_offset;
    let viewport_bottom = scroll_offset + viewport_height;

    for line in lines {
        let height = crate::logic::views::fonts::measure_line_height(ui, line);
        let _rect = embedded_graphics::primitives::Rectangle::new(
            embedded_graphics::prelude::Point::new(0, y as i32),
            embedded_graphics::prelude::Size::new(width, height),
        );
        let line_top = y;
        let line_bottom = y + height;
        // Only include lines that intersect the viewport
        if line_bottom > viewport_top && line_top < viewport_bottom {
            // Adjust rect to be relative to viewport (y - scroll_offset)
            let adjusted_rect = embedded_graphics::primitives::Rectangle::new(
                embedded_graphics::prelude::Point::new(0, (y as i32) - (scroll_offset as i32)),
                embedded_graphics::prelude::Size::new(width, height),
            );
            composed_lines.push(ComposedUiLine {
                line: line.clone(),
                rect: adjusted_rect,
            });
        }
        y += height;
    }

    ComposedForm {
        lines: composed_lines,
        scroll_offset,
        viewport_height,
    }
}

/// Scroll up by one line (returns new offset).
pub fn scroll_up(current_offset: u32, lines: &[ComposedUiLine], _viewport_height: u32) -> u32 {
    // Find the first line that starts at or above the current offset
    if lines.is_empty() || current_offset == 0 {
        return 0;
    }
    // Find the line whose bottom is just above the current offset
    let mut prev_offset = 0u32;
    for line in lines {
        let line_bottom = (line.rect.top_left.y + line.rect.size.height as i32) as u32;
        if line_bottom >= current_offset {
            break;
        }
        prev_offset = line.rect.top_left.y as u32 + 0; // top of this line
    }
    prev_offset
}

/// Scroll down by one line (returns new offset).
pub fn scroll_down(current_offset: u32, lines: &[ComposedUiLine], viewport_height: u32) -> u32 {
    if lines.is_empty() {
        return current_offset;
    }
    // Find the first line whose top is below the current viewport
    let mut next_offset = current_offset;
    for line in lines {
        let line_top = line.rect.top_left.y as u32;
        if line_top > 0 && line_top > current_offset {
            next_offset = line_top;
            break;
        }
    }
    // Clamp so we don't scroll past the last line
    let total_height = lines.last().map(|l| (l.rect.top_left.y + l.rect.size.height as i32) as u32).unwrap_or(0);
    if next_offset + viewport_height > total_height {
        if total_height > viewport_height {
            return total_height - viewport_height;
        } else {
            return 0;
        }
    }
    next_offset
}

/// Draws a line of type Elements using the provided UI context.
fn draw_elements_line(
    elements: &[crate::logic::views::UiLineElement],
    rect: &embedded_graphics::primitives::Rectangle,
    ui: &mut CardworderUi<'_>,
) {
    let mut x = rect.top_left.x;
    let y = rect.top_left.y;
    for element in elements.iter() {
        match element {
            UiLineElement::Icon(ch, font, color) => {
                let _ = ui.draw_text_oneline(
                    *ch,
                    *font,
                    *color,
                    embedded_graphics::prelude::Point::new(x, y),
                    u8g2_fonts::types::VerticalPosition::Center,
                );
                x += ui.font_height(*font) as i32;
            }
            UiLineElement::Text(text, font, vpos, color) => {
                let rect = ui.draw_text_oneline(
                    *text,
                    *font,
                    *color,
                    embedded_graphics::prelude::Point::new(x, y),
                    *vpos,
                );
                if let Some(r) = rect {
                    x += r.size.width as i32;
                }
            }
            UiLineElement::Spacer(pixels) => {
                x += *pixels as i32;
            }
            UiLineElement::Filler => {
                // Filler: do nothing
            }
        }
    }
}

/// Render only visible lines (calls low-level drawing).
pub fn render_visible_lines(
    composed: &ComposedForm,
    ui: &mut CardworderUi<'_>,
) {
    for composed_line in &composed.lines {
        match composed_line.line {
            UiLineType::Elements(ref elements) => {
                draw_elements_line(elements, &composed_line.rect, ui);
            }
            UiLineType::Spacer(_) => {
                // Spacer: nothing to draw
            }
            UiLineType::Line(_, _color) => {
                // Draw a horizontal line across the rect
                // (implement as needed)
            }
        }
    }
} 