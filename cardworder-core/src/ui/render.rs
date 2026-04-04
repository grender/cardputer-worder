//! Compose, scroll, and render logic for scrollable forms in Cardputer UI.

use crate::ui::elements::{UiLineElement, UiLineType};
use crate::ui::cardworder_ui::{CardFont, CardworderUi, ThemeColor, TOP_BAR_HEIGHT};
use crate::ui::framebuffer::CardworderFB;
use embedded_graphics::prelude::{Point, Size};
use embedded_graphics::primitives::Rectangle;

pub struct ComposedForm {
    pub lines: Vec<ComposedUiLine>,
    pub scroll_offset: u32,
    pub viewport_height: u32,
    pub total_content_height: u32,
}

pub struct ComposedUiLine {
    pub line_index: usize,
    pub rect: Rectangle,
}

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
    pub fn new() -> Self {
        Self {
            visible: false,
            width: 4,
            track_height: 0,
            thumb_height: 0,
            thumb_position: 0,
            total_content_height: 0,
            viewport_height: 0,
            scroll_offset: 0,
        }
    }

    pub fn calculate(
        &mut self,
        total_content_height: u32,
        viewport_height: u32,
        scroll_offset: u32,
    ) {
        self.total_content_height = total_content_height;
        self.viewport_height = viewport_height;
        self.scroll_offset = scroll_offset;
        self.visible = total_content_height > viewport_height;

        if self.visible {
            self.track_height = viewport_height;
            let visible_ratio = viewport_height as f32 / total_content_height as f32;
            self.thumb_height = (viewport_height as f32 * visible_ratio) as u32;
            if self.thumb_height < 8 {
                self.thumb_height = 8;
            }
            let scroll_ratio =
                scroll_offset as f32 / (total_content_height - viewport_height) as f32;
            let max_thumb_offset = viewport_height - self.thumb_height;
            self.thumb_position = (scroll_ratio * max_thumb_offset as f32) as u32;
        }
    }

    pub fn get_track_rect(&self, screen_width: u32) -> Rectangle {
        Rectangle::new(
            Point::new((screen_width - self.width) as i32, 0),
            Size::new(self.width, self.track_height),
        )
    }

    pub fn get_thumb_rect(&self, screen_width: u32) -> Rectangle {
        Rectangle::new(
            Point::new((screen_width - self.width) as i32, self.thumb_position as i32),
            Size::new(self.width, self.thumb_height),
        )
    }
}

pub fn compose_scrolled_form<'a, FB: CardworderFB>(
    lines: &[UiLineType<'a>],
    selected_line_idx: usize,
    viewport_height: u32,
    content_top_y: i32,
    ui: &CardworderUi<FB>,
) -> ComposedForm {
    let width = 240u32;
    let mut heights = Vec::with_capacity(lines.len());
    let mut y = 0u32;
    let mut selected_top = 0u32;

    for (idx, line) in lines.iter().enumerate() {
        let h = crate::ui::fonts::measure_line_height(ui, line);
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
                Point::new(
                    0,
                    (y as i32) - (scroll_offset as i32) + content_top_y,
                ),
                Size::new(width, h),
            );
            composed_lines.push(ComposedUiLine { line_index: idx, rect: adjusted_rect });
        }
        y += h;
    }

    ComposedForm { lines: composed_lines, scroll_offset, viewport_height, total_content_height }
}

fn draw_elements_line<FB: CardworderFB>(
    elements: &[UiLineElement<'_>],
    rect: &Rectangle,
    ui: &mut CardworderUi<FB>,
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
                    Point::new(x, y_center),
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
                    Point::new(x, draw_y),
                    *vpos,
                );
                if let Some(r) = text_rect {
                    x += r.size.width as i32;
                }
            }
            UiLineElement::Spacer(pixels) => {
                x += *pixels as i32;
            }
            UiLineElement::Filler => {}
        }
    }
}

pub fn render_scroll_bar<FB: CardworderFB>(
    scroll_bar: &ScrollBar,
    ui: &mut CardworderUi<FB>,
    screen_width: u32,
) {
    if !scroll_bar.visible {
        return;
    }
    let track_rect = scroll_bar.get_track_rect(screen_width);
    let adjusted_track_rect = Rectangle::new(
        Point::new(track_rect.top_left.x, track_rect.top_left.y + TOP_BAR_HEIGHT as i32),
        track_rect.size,
    );
    let track_color = embedded_graphics::pixelcolor::Rgb565::new(20, 20, 20);
    ui.fill_rect(adjusted_track_rect, track_color);

    let thumb_rect = scroll_bar.get_thumb_rect(screen_width);
    let adjusted_thumb_rect = Rectangle::new(
        Point::new(thumb_rect.top_left.x, thumb_rect.top_left.y + TOP_BAR_HEIGHT as i32),
        thumb_rect.size,
    );
    let thumb_color = embedded_graphics::pixelcolor::Rgb565::new(100, 100, 100);
    ui.fill_rect(adjusted_thumb_rect, thumb_color);
}

fn draw_input_field_line<FB: CardworderFB>(
    label: &str,
    value: &str,
    cursor_pos: usize,
    focused: bool,
    rect: &Rectangle,
    ui: &mut CardworderUi<FB>,
) {
    use embedded_graphics::pixelcolor::Rgb565;
    use embedded_graphics::prelude::{RgbColor, WebColors};

    let x = rect.top_left.x + 2;
    let y = rect.top_left.y;
    let width = rect.size.width.saturating_sub(4);

    let label_h = ui.font_height(CardFont::Small) as i32;
    ui.draw_text_oneline(
        label,
        CardFont::Small,
        ThemeColor::Text,
        Point::new(x, y),
        u8g2_fonts::types::VerticalPosition::Top,
    );

    let box_y = y + label_h + 2;
    let box_h = ui.font_height(CardFont::Medium) as i32 + 6;
    let border_color = if focused { Rgb565::CSS_LIGHT_BLUE } else { Rgb565::CSS_GRAY };

    let box_rect = Rectangle::new(Point::new(x, box_y), Size::new(width, box_h as u32));
    ui.fill_rect(box_rect, border_color);
    let inner = Rectangle::new(
        Point::new(x + 1, box_y + 1),
        Size::new(width.saturating_sub(2), (box_h - 2).max(0) as u32),
    );
    ui.fill_rect(inner, Rgb565::new(4, 8, 4));

    let text_x = x + 3;
    let text_y = box_y + 3;
    if !value.is_empty() {
        ui.draw_text_oneline(
            value,
            CardFont::Medium,
            ThemeColor::Text,
            Point::new(text_x, text_y),
            u8g2_fonts::types::VerticalPosition::Top,
        );
    }

    if focused {
        let char_w = ui.font_width(CardFont::Medium) as i32;
        let cursor_x = text_x + (cursor_pos as i32) * char_w;
        let cursor_rect = Rectangle::new(
            Point::new(cursor_x, box_y + 2),
            Size::new(2, (box_h - 4).max(1) as u32),
        );
        ui.fill_rect(cursor_rect, Rgb565::WHITE);
    }
}

pub fn render_visible_lines<FB: CardworderFB>(
    composed: &ComposedForm,
    lines: &[UiLineType<'_>],
    ui: &mut CardworderUi<FB>,
) {
    for composed_line in &composed.lines {
        match &lines[composed_line.line_index] {
            UiLineType::Elements(elements) => {
                draw_elements_line(elements, &composed_line.rect, ui);
            }
            UiLineType::InputField { label, value, cursor_pos, focused } => {
                draw_input_field_line(
                    label,
                    value.as_str(),
                    *cursor_pos,
                    *focused,
                    &composed_line.rect,
                    ui,
                );
            }
            UiLineType::AutoText { text, color, max_width } => {
                ui.draw_text_auto(
                    text,
                    *color,
                    composed_line.rect.top_left.x + 4,
                    composed_line.rect.top_left.y,
                    *max_width,
                );
            }
            UiLineType::Spacer(_) => {}
            UiLineType::Line(_, _color) => {}
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
