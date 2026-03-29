//! HAL test: scroll exploration. Fills entire ST7789V2 RAM (240×320) with
//! gradients and lets you change scroll offset with keyboard.

use core::fmt::Write;

use cardworder::cardputer_hal::screen::cardputer_screen::CardputerScreen;
use cardworder::cardputer_hal::input::keyboard_io::{CardputerKeyboard, KeyEvent, Scancode};
use cardworder::cardputer_hal::input::keyboard::{InputState, InputLanguage, PressedSymbol};
use cardworder::ResultExt;
use display_interface::{DataFormat, WriteOnlyDataCommand};
use embedded_graphics::pixelcolor::Rgb565;
use embedded_graphics::prelude::{DrawTarget, IntoStorage, Point, RgbColor, Size, WebColors};
use embedded_graphics::primitives::Rectangle;
use esp_idf_hal::delay::FreeRtos;
use esp_idf_hal::gpio::{Output, PinDriver, Pull};
use esp_idf_svc::hal::peripherals::Peripherals;
use mipidsi::dcs::{SetColumnAddress, SetPageAddress, SetScrollStart, WriteMemoryStart};
use u8g2_fonts::types::{FontColor, VerticalPosition};
use u8g2_fonts::{fonts, FontRenderer};

fn main() {
    esp_idf_svc::sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    log::info!("hal_test_scroll: === start ===");
    let peripherals = Peripherals::take().unwrap_or_log("error get peripherals");

    // Build display
    let mut screen = CardputerScreen::build(
        Rgb565::CSS_BLACK,
        peripherals.spi2,
        peripherals.pins.gpio36,
        peripherals.pins.gpio35,
        peripherals.pins.gpio37,
        peripherals.pins.gpio34,
        peripherals.pins.gpio33,
        peripherals.pins.gpio38,
    );
    screen.backlight_on().ok();

    // Get raw display for direct DCS access
    let (mut display, _framebuffer) = screen.into_parts();

    // Build keyboard
    let mux_pins: [PinDriver<'_, Output>; 3] = [
        PinDriver::output(peripherals.pins.gpio8.degrade_output()).unwrap(),
        PinDriver::output(peripherals.pins.gpio9.degrade_output()).unwrap(),
        PinDriver::output(peripherals.pins.gpio11.degrade_output()).unwrap(),
    ];
    let column_pins = [
        PinDriver::input(peripherals.pins.gpio13.degrade_input_output(), Pull::Up).unwrap(),
        PinDriver::input(peripherals.pins.gpio15.degrade_input_output(), Pull::Up).unwrap(),
        PinDriver::input(peripherals.pins.gpio3.degrade_input_output(), Pull::Up).unwrap(),
        PinDriver::input(peripherals.pins.gpio4.degrade_input_output(), Pull::Up).unwrap(),
        PinDriver::input(peripherals.pins.gpio5.degrade_input_output(), Pull::Up).unwrap(),
        PinDriver::input(peripherals.pins.gpio6.degrade_input_output(), Pull::Up).unwrap(),
        PinDriver::input(peripherals.pins.gpio7.degrade_input_output(), Pull::Up).unwrap(),
    ];
    let mut keyboard = CardputerKeyboard::new(mux_pins, column_pins);
    keyboard.init();

    let font = FontRenderer::new::<fonts::u8g2_font_5x8_t_cyrillic>();

    // Fill entire 240×320 display RAM with gradients + row numbers
    log::info!("hal_test_scroll: filling 240x320 RAM...");
    fill_full_ram(&mut display, &font);
    log::info!("hal_test_scroll: RAM filled");

    // Main loop: keyboard controls scroll offset
    let mut offset: u16 = 0;
    let mut input_state = InputState {
        ctrl_pressed: false, shift_pressed: false, opt_pressed: false,
        alt_pressed: false, fn_pressed: false,
        fn_locked: true, shift_locked: false,
        fn_last_release_us: 0, shift_last_release_us: 0,
        lang: InputLanguage::En,
    };

    log::info!("hal_test_scroll: offset={} — arrows=±1, shift+arrows=±10, hold=repeat", offset);

    let mut held_direction: i16 = 0; // -1 = left/up, +1 = right/down, 0 = none
    let mut last_repeat_us: u64 = 0;
    let repeat_delay_us: u64 = 80_000; // 80ms repeat rate

    loop {
        let now_us = unsafe { esp_idf_svc::sys::esp_timer_get_time() as u64 };

        if let Some((event, scancode)) = keyboard.read_events() {
            if let Some(pressed) = input_state.eat_keys(event, scancode) {
                match (event, &pressed) {
                    (KeyEvent::Pressed, PressedSymbol::ArrowUp | PressedSymbol::ArrowLeft) => {
                        held_direction = -1;
                        last_repeat_us = now_us;
                    }
                    (KeyEvent::Pressed, PressedSymbol::ArrowDown | PressedSymbol::ArrowRight) => {
                        held_direction = 1;
                        last_repeat_us = now_us;
                    }
                    _ => {}
                }
                // Apply on press
                if matches!(event, KeyEvent::Pressed) {
                    let step: i16 = if input_state.shift_active() { 10 } else { 1 };
                    match pressed {
                        PressedSymbol::ArrowUp | PressedSymbol::ArrowLeft => {
                            offset = ((offset as i16 - step + 320) % 320) as u16;
                        }
                        PressedSymbol::ArrowDown | PressedSymbol::ArrowRight => {
                            offset = ((offset as i16 + step) % 320) as u16;
                        }
                        _ => {}
                    }
                    unsafe { display.screen.dcs().write_command(SetScrollStart::new(offset)).unwrap(); }
                    log::info!("hal_test_scroll: offset={}", offset);
                }
            }
            // Stop repeat on any release of arrow-related keys
            if matches!(event, KeyEvent::Released) && matches!(scancode,
                Scancode::Semicolon | Scancode::Slash | Scancode::Comma | Scancode::Period) {
                held_direction = 0;
            }
        }

        // Key repeat while held
        if held_direction != 0 && (now_us - last_repeat_us) >= repeat_delay_us {
            let step: i16 = if input_state.shift_active() { 10 } else { 1 };
            offset = ((offset as i16 + held_direction * step + 320) % 320) as u16;
            unsafe { display.screen.dcs().write_command(SetScrollStart::new(offset)).unwrap(); }
            log::info!("hal_test_scroll: offset={}", offset);
            last_repeat_us = now_us;
        }

        FreeRtos::delay_ms(5);
    }
}

/// Fill ENTIRE chip memory (240 cols × 320 rows, no offset) with monotonic gradient + 10×10 grid.
fn fill_full_ram(display: &mut cardworder::cardputer_hal::screen::display::CardputerDisplay<'_>, _font: &FontRenderer) {
    // Native chip framebuffer: 240 columns × 320 rows
    // Write to ALL of it: cols 0-239, rows 0-319
    let chunk_rows = 16u16;

    for chunk_start in (0..320u16).step_by(chunk_rows as usize) {
        let chunk_end = (chunk_start + chunk_rows - 1).min(319);
        let rows = (chunk_end - chunk_start + 1) as usize;
        let mut pixels: Vec<u16> = Vec::with_capacity(240 * rows);

        for row in chunk_start..=chunk_end {
            for col in 0..240u16 {
                // Monotonic diagonal gradient across full 240×320
                let t = (row as f32 + col as f32) / (320.0 + 240.0);
                let mut r = (31.0 * (1.0 - t) * 0.6) as u8;
                let mut g = (63.0 * t) as u8;
                let mut b = (31.0 * (0.3 + 0.7 * t)) as u8;

                // 10×10 grid: invert color on grid lines
                if row % 10 == 0 || col % 10 == 0 {
                    r = 31 - r;
                    g = 63 - g;
                    b = 31 - b;
                }

                let color = Rgb565::new(r, g, b);
                pixels.push(color.into_storage().swap_bytes());
            }
        }

        // Write to FULL chip address space: cols 0-239, rows chunk
        unsafe {
            display.screen.dcs().write_command(SetColumnAddress::new(0, 239)).unwrap();
            display.screen.dcs().write_command(SetPageAddress::new(chunk_start, chunk_end)).unwrap();
            display.screen.dcs().write_command(WriteMemoryStart).unwrap();
            let bytes: &[u8] = core::slice::from_raw_parts(
                pixels.as_ptr() as *const u8,
                pixels.len() * 2,
            );
            display.screen.dcs().di.send_data(DataFormat::U8(bytes)).unwrap();
        }
    }
}
