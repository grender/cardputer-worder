use embedded_graphics::pixelcolor::Rgb565;
use embedded_graphics::prelude::{Point, RgbColor};
use esp_idf_hal::delay::FreeRtos;
use u8g2_fonts::types::VerticalPosition;

use crate::cardputer_hal::input::keyboard_io::CardputerKeyboard;
use crate::ui::cardworder_ui::{CardFont, CardworderUi, ThemeColor};

pub fn run_splash(ui: &mut CardworderUi, keyboard: &mut CardputerKeyboard<'_>) {
    log::info!("boot: splash animation");

    let title = "CardWorder";
    let title_x = 70i32;
    let title_y = 47i32;
    let subtitle_x = 69i32;
    let subtitle_y = 78i32;
    let char_w = 10i32;

    // Phase 1: Clear screen
    ui.clear(Rgb565::BLACK);
    ui.flip_buffer();

    // Phase 2: Letter-by-letter reveal
    for (i, ch) in title.chars().enumerate() {
        if keyboard.read_events().is_some() {
            let remaining = &title[title.char_indices().nth(i).map(|(b,_)|b).unwrap_or(title.len())..];
            let x = title_x + (i as i32) * char_w;
            ui.draw_text_oneline(remaining, CardFont::XLarge, ThemeColor::Selected,
                Point::new(x, title_y), VerticalPosition::Top);
            ui.flip_buffer();
            return;
        }
        let x = title_x + (i as i32) * char_w;
        let mut buf = [0u8; 4];
        let s: &str = ch.encode_utf8(&mut buf);
        ui.draw_text_oneline(&*s, CardFont::XLarge, ThemeColor::Selected,
            Point::new(x, title_y), VerticalPosition::Top);
        ui.flip_buffer();
        FreeRtos::delay_ms(60);
    }

    // Phase 3: Subtitle fade in
    for step in 0..10u32 {
        if keyboard.read_events().is_some() { break; }
        let b = ((step + 1) * 25).min(255) as u8;
        let color = Rgb565::new(b >> 3, b >> 2, b >> 3);
        ui.draw_text_oneline(
            "Spaced Repetition", CardFont::Medium,
            ThemeColor::Color(color),
            Point::new(subtitle_x, subtitle_y), VerticalPosition::Top,
        );
        ui.flip_buffer();
        FreeRtos::delay_ms(40);
    }

    // Phase 4: Hold
    for _ in 0..60 {
        if keyboard.read_events().is_some() { break; }
        FreeRtos::delay_ms(10);
    }
}
