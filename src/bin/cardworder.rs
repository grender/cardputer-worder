use cardworder::cardputer_hal::cardputer_hal::CardputerHal;
use cardworder::cardputer_hal::input::keyboard::PressedSymbol;
use cardworder::logic::view_manager::{ViewManager};
use cardworder::logic::views::start::StartView;
use cardworder::ui::cardworder_ui::CardworderUi;
use cardworder::ResultExt;
use embedded_graphics::prelude::{RgbColor, WebColors};
use esp_idf_svc::eventloop::EspSystemEventLoop;
use esp_idf_svc::hal::prelude::Peripherals;

use embedded_graphics::pixelcolor::Rgb565;
use esp_idf_svc::sntp::{EspSntp, SyncStatus};
use esp_idf_sys::{setenv, tzset};

fn main() {
    esp_idf_svc::sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    log::info!("Start the app");

    let peripherals = Peripherals::take().unwrap_or_log("error get peripherals");
    let sysloop = EspSystemEventLoop::take().unwrap_or_log("error init event loop");
    let mut hal = CardputerHal::new(peripherals, sysloop.clone());

    let screen = hal.take_screen();
    let ui = CardworderUi::build(screen);

    let mut view_manager = ViewManager::new(hal, ui, Box::new(StartView{}));

    loop {
        view_manager.loop_logic();
    }

    /*
    
    loop {
        cardputer_hal.update_keyboard_state();
        let keyboard_state = &cardputer_hal.keyboard_state;
        match (keyboard_state.input_state.opt_pressed, keyboard_state.key) {
            (true, Some((KeyEvent::Pressed, Scancode::F))) => {
                ui.show_fps = !ui.show_fps;
            }
            _ => {}
        }

        match keyboard_state.pressed {
            Some((KeyEvent::Pressed, PressedSymbol::ArrowDown)) => {
                ui.clear(Rgb565::CSS_DARK_GRAY);
                is_changed = true;
            }
            Some((KeyEvent::Released, PressedSymbol::ArrowDown)) => {
                ui.backlight_off();
            }
            Some((KeyEvent::Pressed, PressedSymbol::ArrowUp)) => {
                ui.clear(Rgb565::CSS_WHEAT);
                ui.backlight_on();
                is_changed = true;
            }
            Some((KeyEvent::Pressed, PressedSymbol::Char(' '))) => {
                is_bold = !is_bold;
                ui.clear(Rgb565::CSS_RED);
                is_changed = true;
            }
            _ => {}
        }

        if is_changed {
            ui.draw_long_text(is_bold);
        }
        ui.draw_top_line(&keyboard_state.input_state, &keyboard_state.pressed);
        ui.flip_buffer();
    }
    */
}
