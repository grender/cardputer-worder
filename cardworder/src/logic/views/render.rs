//! Compose, scroll, and render logic for scrollable forms in Cardputer UI.

use crate::logic::views::{UiLineElement, UiLineType};
use crate::ui::cardworder_ui::{CardworderUi, TOP_BAR_HEIGHT};
use embedded_graphics::prelude::{Point, Size};
use embedded_graphics::primitives::Rectangle;

/// State for a composed form, including scroll offset.
pub struct ComposedForm {
    pub lines: Vec<ComposedUiLine>,
    pub scroll_offset: u32,
    pub viewport_height: u32,
    pub total_content_height: u32,
}

/// A composed line: index into the original `lines` slice plus screen rectangle.
pub struct ComposedUiLine {
    pub line_index: usize,
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

/// Compose lines in one pass: measure heights once, scroll so `selected_line_idx` is at the top
/// of the viewport when possible, and return visible rows as indices into `lines`.
/// `content_top_y` is the screen y where the content area starts (e.g. below the top bar).
pub fn compose_scrolled_form(
    lines: &[UiLineType],
    selected_line_idx: usize,
    viewport_height: u32,
    content_top_y: i32,
    ui: &CardworderUi,
) -> ComposedForm {
    let width = 240u32;
    let mut heights = Vec::with_capacity(lines.len());
    let mut y = 0u32;
    let mut selected_top = 0u32;

    for (idx, line) in lines.iter().enumerate() {
        let h = crate::logic::views::fonts::measure_line_height(ui, line);
        if idx == selected_line_idx {
            selected_top = y;
        }
        heights.push(h);
        y += h;
    }

    let total_content_height = y;
    let max_scroll = total_content_height.saturating_sub(viewport_height);
    let scroll_offset = selected_top.min(max_scroll);
    let viewport_top = scroll_offset;
    let viewport_bottom = scroll_offset.saturating_add(viewport_height);

    let mut composed_lines = Vec::new();
    y = 0;
    for (idx, _line) in lines.iter().enumerate() {
        let h = heights[idx];
        let line_top = y;
        let line_bottom = y + h;
        if line_bottom > viewport_top && line_top < viewport_bottom {
            let adjusted_rect = Rectangle::new(
                Point::new(0, (y as i32) - (scroll_offset as i32) + content_top_y),
                Size::new(width, h),
            );
            composed_lines.push(ComposedUiLine {
                line_index: idx,
                rect: adjusted_rect,
            });
        }
        y += h;
    }

    ComposedForm {
        lines: composed_lines,
        scroll_offset,
        viewport_height,
        total_content_height,
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
/// Uses the vertical center of the line rect for Center-aligned elements so glyphs
/// do not extend above the content area (e.g. into the top bar).
fn draw_elements_line(
    elements: &[crate::logic::views::UiLineElement],
    rect: &embedded_graphics::primitives::Rectangle,
    ui: &mut CardworderUi<'_>,
) {
    let mut x = rect.top_left.x;
    let line_top_y = rect.top_left.y;
    let line_height = rect.size.height as i32;
    let y_center = line_top_y + line_height / 2;
    for element in elements.iter() {
        match element {
            UiLineElement::Icon(ch, font, color) => {
                let _ = ui.draw_text_oneline(
                    *ch,
                    *font,
                    *color,
                    embedded_graphics::prelude::Point::new(x, y_center),
                    u8g2_fonts::types::VerticalPosition::Center,
                );
                x += ui.font_height(*font) as i32;
            }
            UiLineElement::Text(text, font, vpos, color) => {
                let draw_y = match vpos {
                    u8g2_fonts::types::VerticalPosition::Center => y_center,
                    u8g2_fonts::types::VerticalPosition::Top => line_top_y,
                    u8g2_fonts::types::VerticalPosition::Bottom => line_top_y + line_height,
                    u8g2_fonts::types::VerticalPosition::Baseline => y_center,
                };
                let text_rect = ui.draw_text_oneline(
                    *text,
                    *font,
                    *color,
                    embedded_graphics::prelude::Point::new(x, draw_y),
                    *vpos,
                );
                if let Some(r) = text_rect {
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
    let adjusted_track_rect = Rectangle::new(
        Point::new(
            track_rect.top_left.x,
            track_rect.top_left.y + TOP_BAR_HEIGHT as i32,
        ),
        track_rect.size,
    );
    // Use a dark color for the track
    let track_color = embedded_graphics::pixelcolor::Rgb565::new(20, 20, 20);
    ui.fill_rect(adjusted_track_rect, track_color);

    let thumb_rect = scroll_bar.get_thumb_rect(screen_width);
    let adjusted_thumb_rect = Rectangle::new(
        Point::new(
            thumb_rect.top_left.x,
            thumb_rect.top_left.y + TOP_BAR_HEIGHT as i32,
        ),
        thumb_rect.size,
    );
    // Use a lighter color for the thumb
    let thumb_color = embedded_graphics::pixelcolor::Rgb565::new(100, 100, 100);
    ui.fill_rect(adjusted_thumb_rect, thumb_color);
}

/// Render only visible lines (calls low-level drawing).
pub fn render_visible_lines(
    composed: &ComposedForm,
    lines: &[UiLineType],
    ui: &mut CardworderUi<'_>,
) {
    for composed_line in &composed.lines {
        match &lines[composed_line.line_index] {
            UiLineType::Elements(elements) => {
                draw_elements_line(elements, &composed_line.rect, ui);
            }
            UiLineType::Spacer(_) => {}
            UiLineType::Line(_, _color) => {
                // Draw a horizontal line across the rect
                // (implement as needed)
            }
        }
    }

    let mut scroll_bar = ScrollBar::new();
    scroll_bar.calculate(
        composed.total_content_height,
        composed.viewport_height,
        composed.scroll_offset,
    );
    render_scroll_bar(&scroll_bar, ui, 240);
} 