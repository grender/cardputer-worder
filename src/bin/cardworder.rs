use cardworder::cardputer_hal::input::keyboard::InputLanguage;
use cardworder::cardputer_hal::input::keyboard::InputState;
use cardworder::cardputer_hal::input::keyboard::PressedSymbol;
use cardworder::cardputer_hal::input::keyboard_io::{CardputerKeyboard, Key, KeyEvent};
use cardworder::cardputer_hal::screen::cardputer_screen::CardputerScreen;
use cardworder::cardputer_hal::sd::cardputer_sd::CardputerSd;
use cardworder::cardputer_hal::wifi::wifi::{self};
use cardworder::ui::cardworder_ui::CardworderUi;
use embedded_graphics::prelude::{RgbColor, WebColors};
use esp_idf_hal::gpio::{self, IOPin, Output, OutputPin, PinDriver};
use esp_idf_svc::eventloop::EspSystemEventLoop;
use esp_idf_svc::hal::prelude::Peripherals;

use embedded_graphics::pixelcolor::Rgb565;
use esp_idf_svc::sntp::{EspSntp, SyncStatus};
use esp_idf_svc::wifi::EspWifi;
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

    let display = CardputerScreen::build(
        Rgb565::CSS_BLACK,
        peripherals.spi2,
        peripherals.pins.gpio36,
        peripherals.pins.gpio35,
        peripherals.pins.gpio37,
        peripherals.pins.gpio34,
        peripherals.pins.gpio33,
        peripherals.pins.gpio38,
    );

    let mut ui = CardworderUi::build(display);

    let sd = CardputerSd::build(
        peripherals.spi3,
        peripherals.pins.gpio40,
        peripherals.pins.gpio39,
        peripherals.pins.gpio14,
        peripherals.pins.gpio12,
    );

    let mux_pins: [PinDriver<'_, gpio::AnyOutputPin, Output>; 3] = [
        PinDriver::output(peripherals.pins.gpio8.downgrade_output()).unwrap(),
        PinDriver::output(peripherals.pins.gpio9.downgrade_output()).unwrap(),
        PinDriver::output(peripherals.pins.gpio11.downgrade_output()).unwrap(),
    ];

    let column_pins = [
        PinDriver::input(peripherals.pins.gpio13.downgrade()).unwrap(),
        PinDriver::input(peripherals.pins.gpio15.downgrade()).unwrap(),
        PinDriver::input(peripherals.pins.gpio3.downgrade()).unwrap(),
        PinDriver::input(peripherals.pins.gpio4.downgrade()).unwrap(),
        PinDriver::input(peripherals.pins.gpio5.downgrade()).unwrap(),
        PinDriver::input(peripherals.pins.gpio6.downgrade()).unwrap(),
        PinDriver::input(peripherals.pins.gpio7.downgrade()).unwrap(),
    ];

    let sysloop = EspSystemEventLoop::take().unwrap_or_log("error init event loop");
    let esp_wifi =
        EspWifi::new(peripherals.modem, sysloop.clone(), None).unwrap_or_log("error init wifi");
    let mut wifi = wifi::CardWorderWifi::new(esp_wifi, sd);

    let mut keyboard = CardputerKeyboard::new(mux_pins, column_pins);
    keyboard.init();

    let mut input_state = InputState {
        ctrl_pressed: false,
        shift_pressed: false,
        opt_pressed: false,
        alt_pressed: false,
        fn_pressed: false,
        lang: InputLanguage::En,
    };

    let mut is_bold = false;
    let mut is_changed = true;

    wifi.create_file_if_non_exists(
        heapless::String::try_from("John24").unwrap(),
        heapless::String::try_from("52525252").unwrap(),
    );

    ui.draw_starting_line("Starting Wifi...", Rgb565::BLACK, Rgb565::WHITE);
    ui.flip_buffer();

    wifi.connect().unwrap_or_log("error connecting to wifi");

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
        let key = keyboard.read_events();

        let pressed = match key {
            Some((event, key)) => input_state.eat_keys(event, key).map(|f| (event, f)),
            None => None,
        };

        match (input_state.opt_pressed, key) {
            (true, Some((KeyEvent::Pressed, Key::F))) => {
                ui.show_fps = !ui.show_fps;
            }
            _ => {}
        }

        match pressed {
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
        ui.draw_top_line(&input_state, &pressed);
        ui.flip_buffer();
    }
}
