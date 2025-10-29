//! Compose, scroll, and render logic for scrollable forms in Cardputer UI.

use crate::logic::views::{UiLineType, UiLineElement};
use crate::logic::views::fonts::measure_line_height;
use embedded_graphics::primitives::Rectangle;
use embedded_graphics::prelude::{Size, Point};
use crate::ui::cardworder_ui::CardworderUi;

/// State for a composed form, including scroll offset.
pub struct ComposedForm<'a> {
    pub lines: Vec<ComposedUiLine<'a>>,
    pub scroll_offset: u32,
    pub viewport_height: u32,
    pub total_content_height: u32,
}

/// A composed line with its rectangle.
pub struct ComposedUiLine<'a> {
    pub line: UiLineType<'a>,
    pub rect: embedded_graphics::primitives::Rectangle,
}

/// Scroll bar configuration and state
/// 
/// The scroll bar provides visual feedback about:
/// - How much content is visible vs. total content
/// - Current scroll position within the content
/// - Whether scrolling is possible (content > viewport)
/// 
/// The scroll bar appears on the right side of the screen and shows:
/// - A dark track (background) representing the total viewport height
/// - A lighter thumb (slider) whose size and position indicate:
///   - Size: proportional to visible content (smaller thumb = more content hidden)
///   - Position: current scroll offset relative to total scrollable area
pub struct ScrollBar {
    pub visible: bool,
    pub width: u32,
    pub track_height: u32,
    pub thumb_height: u32,
    pub thumb_position: u32,
    pub total_content_height: u32,
    pub viewport_height: u32,
    pub scroll_offset: u32,
}

impl ScrollBar {
    /// Create a new scroll bar with default configuration
    pub fn new() -> Self {
        Self {
            visible: false,
            width: 4, // 4 pixels wide
            track_height: 0,
            thumb_height: 0,
            thumb_position: 0,
            total_content_height: 0,
            viewport_height: 0,
            scroll_offset: 0,
        }
    }

    /// Calculate scroll bar dimensions and position based on content and viewport
    pub fn calculate(&mut self, total_content_height: u32, viewport_height: u32, scroll_offset: u32) {
        self.total_content_height = total_content_height;
        self.viewport_height = viewport_height;
        self.scroll_offset = scroll_offset;
        
        // Only show scroll bar if content is taller than viewport
        self.visible = total_content_height > viewport_height;
        
        if self.visible {
            self.track_height = viewport_height;
            
            // Calculate thumb height (proportional to visible content)
            let visible_ratio = viewport_height as f32 / total_content_height as f32;
            self.thumb_height = (viewport_height as f32 * visible_ratio) as u32;
            
            // Ensure minimum thumb height for usability
            if self.thumb_height < 8 {
                self.thumb_height = 8;
            }
            
            // Calculate thumb position based on scroll offset
            let scroll_ratio = scroll_offset as f32 / (total_content_height - viewport_height) as f32;
            let max_thumb_offset = viewport_height - self.thumb_height;
            self.thumb_position = (scroll_ratio * max_thumb_offset as f32) as u32;
        }
    }

    /// Get the scroll bar rectangle for rendering
    pub fn get_track_rect(&self, screen_width: u32) -> Rectangle {
        Rectangle::new(
            Point::new((screen_width - self.width) as i32, 0),
            Size::new(self.width, self.track_height),
        )
    }

    /// Get the scroll bar thumb rectangle for rendering
    pub fn get_thumb_rect(&self, screen_width: u32) -> Rectangle {
        Rectangle::new(
            Point::new((screen_width - self.width) as i32, self.thumb_position as i32),
            Size::new(self.width, self.thumb_height),
        )
    }
}

/// Compose the form: calculate line heights, assign rectangles, and return visible lines.
pub fn compose_form<'a>(
    lines: &[UiLineType<'a>],
    scroll_offset: u32,
    viewport_height: u32,
    ui: &crate::ui::cardworder_ui::CardworderUi,
) -> ComposedForm<'a> {
    let mut composed_lines = Vec::new();
    let mut y = 12u32; // Start below the top line zone (0-11)
    let width = 240; // Assume fixed width for now
    let viewport_top = scroll_offset;
    let viewport_bottom = scroll_offset + viewport_height;

    // First pass: calculate total content height
    let mut total_content_height = 0u32;
    for line in lines {
        let height = crate::logic::views::fonts::measure_line_height(ui, line);
        total_content_height += height;
    }

    // Second pass: compose visible lines
    y = 12; // Start below the top line zone
    for line in lines {
        let height = crate::logic::views::fonts::measure_line_height(ui, line);
        let rect = embedded_graphics::primitives::Rectangle::new(
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
        total_content_height,
    }
}

/// Scroll up by one line (returns new offset).
pub fn scroll_up(current_offset: u32, lines: &[ComposedUiLine], viewport_height: u32) -> u32 {
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
                    u8g2_fonts::types::VerticalPosition::Top,
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
            UiLineElement::InputText(text, font, color, cursor_pos) => {
                let text_str = text.as_str();
                let text_rect = ui.draw_text_oneline(
                    text_str,
                    *font,
                    *color,
                    embedded_graphics::prelude::Point::new(x, y),
                    u8g2_fonts::types::VerticalPosition::Top,
                );
                
                // Calculate cursor position and draw it
                if let Some(r) = text_rect {
                    let cursor_x = x + ui.measure_text_width(text_str, *font, *cursor_pos);
                    ui.draw_cursor(
                        embedded_graphics::prelude::Point::new(cursor_x, y),
                        ui.font_height(*font),
                        *color,
                    );
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

/// Render the scroll bar if it's visible
pub fn render_scroll_bar(
    scroll_bar: &ScrollBar,
    ui: &mut CardworderUi<'_>,
    screen_width: u32,
) {
    if !scroll_bar.visible {
        return;
    }

    // Draw scroll bar track (background) - start below top line zone
    let track_rect = scroll_bar.get_track_rect(screen_width);
    // Adjust track position to start below top line (y=12)
    let adjusted_track_rect = Rectangle::new(
        Point::new(track_rect.top_left.x, track_rect.top_left.y + 12),
        track_rect.size,
    );
    // Use a dark color for the track
    let track_color = embedded_graphics::pixelcolor::Rgb565::new(20, 20, 20);
    ui.fill_rect(adjusted_track_rect, track_color);

    // Draw scroll bar thumb (the draggable part) - start below top line zone
    let thumb_rect = scroll_bar.get_thumb_rect(screen_width);
    // Adjust thumb position to start below top line (y=12)
    let adjusted_thumb_rect = Rectangle::new(
        Point::new(thumb_rect.top_left.x, thumb_rect.top_left.y + 12),
        thumb_rect.size,
    );
    // Use a lighter color for the thumb
    let thumb_color = embedded_graphics::pixelcolor::Rgb565::new(100, 100, 100);
    ui.fill_rect(adjusted_thumb_rect, thumb_color);
}

/// Render only visible lines (calls low-level drawing).
pub fn render_visible_lines(
    composed: &ComposedForm,
    ui: &mut CardworderUi<'_>,
) {
    // Render the content lines
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

    // Render the scroll bar - account for top line zone (12 pixels)
    let mut scroll_bar = ScrollBar::new();
    let adjusted_viewport_height = composed.viewport_height.saturating_sub(12); // Subtract top line height
    scroll_bar.calculate(
        composed.total_content_height,
        adjusted_viewport_height,
        composed.scroll_offset,
    );
    render_scroll_bar(&scroll_bar, ui, 240); // 240 is the screen width
} 