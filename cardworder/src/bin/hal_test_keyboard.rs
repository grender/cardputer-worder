//! HAL test: keyboard matrix only. Pin wiring matches `CardputerHal::new`.

use cardworder::cardputer_hal::input::keyboard_io::CardputerKeyboard;
use cardworder::ResultExt;
use esp_idf_hal::delay::FreeRtos;
use esp_idf_hal::gpio::{Output, PinDriver, Pull};
use esp_idf_svc::hal::peripherals::Peripherals;

fn main() {
    esp_idf_svc::sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    log::info!("hal_test_keyboard: === start ===");
    log::info!("hal_test_keyboard: boot — Peripherals::take");
    let peripherals = Peripherals::take().unwrap_or_log("error get peripherals");
    log::info!("hal_test_keyboard: peripherals ok");

    log::info!("hal_test_keyboard: mux outputs gpio8, gpio9, gpio11");
    let mux_pins: [PinDriver<'_, Output>; 3] = [
        PinDriver::output(peripherals.pins.gpio8.degrade_output()).unwrap(),
        PinDriver::output(peripherals.pins.gpio9.degrade_output()).unwrap(),
        PinDriver::output(peripherals.pins.gpio11.degrade_output()).unwrap(),
    ];
    log::info!("hal_test_keyboard: mux pins — done");

    log::info!("hal_test_keyboard: column inputs gpio13,15,3,4,5,6,7 (pull-up)");
    let column_pins = [
        PinDriver::input(peripherals.pins.gpio13.degrade_input_output(), Pull::Up).unwrap(),
        PinDriver::input(peripherals.pins.gpio15.degrade_input_output(), Pull::Up).unwrap(),
        PinDriver::input(peripherals.pins.gpio3.degrade_input_output(), Pull::Up).unwrap(),
        PinDriver::input(peripherals.pins.gpio4.degrade_input_output(), Pull::Up).unwrap(),
        PinDriver::input(peripherals.pins.gpio5.degrade_input_output(), Pull::Up).unwrap(),
        PinDriver::input(peripherals.pins.gpio6.degrade_input_output(), Pull::Up).unwrap(),
        PinDriver::input(peripherals.pins.gpio7.degrade_input_output(), Pull::Up).unwrap(),
    ];
    log::info!("hal_test_keyboard: column pins — done");

    log::info!("hal_test_keyboard: CardputerKeyboard::new + init");
    let mut keyboard = CardputerKeyboard::new(mux_pins, column_pins);
    keyboard.init();
    log::info!("hal_test_keyboard: keyboard ready — poll loop (log keys when pressed; heartbeat every ~2.5s)");

    let mut iteration = 0u32;
    loop {
        log::info!("hal_test_keyboard: loop iteration {}", iteration);
        FreeRtos::delay_ms(50);
        iteration += 1;

        let keys = keyboard.read_keys();
        if !keys.is_empty() {
            log::info!("hal_test_keyboard: read_keys -> {:?}", keys);
        }

        if iteration % 50 == 0 {
            log::info!("hal_test_keyboard: heartbeat iteration {}", iteration);
        }
    }
}
