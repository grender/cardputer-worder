use cardworder::cardputer_hal::cardputer_hal::CardputerHal;
use cardworder::cardputer_hal::input::keyboard::PressedSymbol;
use cardworder::cardputer_hal::input::keyboard_io::{Scancode, KeyEvent};
use cardworder::ui::cardworder_ui::CardworderUi;
use embedded_graphics::prelude::{RgbColor, WebColors};
use esp_idf_svc::eventloop::EspSystemEventLoop;
use esp_idf_svc::hal::prelude::Peripherals;

use embedded_graphics::pixelcolor::Rgb565;
use esp_idf_svc::sntp::{EspSntp, SyncStatus};
use esp_idf_sys::{setenv, tzset};

pub trait ResultExt<R, E> {
    fn unwrap_or_log(self, message: &str) -> R;
}

impl<R, E: core::fmt::Debug> ResultExt<R, E> for Result<R, E> {
    fn unwrap_or_log(self, message: &str) -> R {
        match self {
            Ok(t) => t,
            Err(e) => {
                log::error!("error: {} {:?}", message, e);
                loop {}
            }
        }
    }
}

fn main() {
    esp_idf_svc::sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    log::info!("Start the app");

    let peripherals = Peripherals::take().unwrap_or_log("error get peripherals");
    let sysloop = EspSystemEventLoop::take().unwrap_or_log("error init event loop");
    let mut cardputer_hal = CardputerHal::new(peripherals, sysloop.clone());

    let screen = cardputer_hal.take_screen();
    let mut ui = CardworderUi::build(screen);

    let mut is_bold = false;
    let mut is_changed = true;

    cardputer_hal.create_wifi_file_if_non_exists(
        heapless::String::try_from("John24").unwrap(),
        heapless::String::try_from("52525252").unwrap(),
    ).unwrap_or_log("error create wifi file");

    let wifi_config = cardputer_hal.load_wifi_config().unwrap_or_log("error load wifi config");
    

    ui.draw_starting_line("Starting Wifi...", Rgb565::BLACK, Rgb565::WHITE);
    ui.flip_buffer();

    cardputer_hal.connect_wifi(wifi_config).unwrap_or_log("error connecting to wifi");

    unsafe {
        let env_tz = b"TZ\0";
        let tz = b"GMT-3\0";
        // TODO: move to a separate file
        setenv(env_tz.as_ptr() as *const i8, tz.as_ptr() as *const i8, 1);
        tzset();
        // let tz = getenv(env_tz.as_ptr() as *const i8);
        // let tz_str = CStr::from_ptr(tz).to_str().unwrap();
        // log::info!("tz: {:?}", tz_str);
    }

    ui.draw_starting_line("Starting NTP...", Rgb565::BLACK, Rgb565::WHITE);
    ui.flip_buffer();

    let ntp = EspSntp::new_default().unwrap();

    ui.draw_starting_line("Awaiting NTP...", Rgb565::BLACK, Rgb565::WHITE);
    ui.flip_buffer();

    while ntp.get_sync_status() != SyncStatus::Completed {}

    ui.draw_starting_line("Got NTP!", Rgb565::BLACK, Rgb565::WHITE);
    ui.flip_buffer();

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
}
