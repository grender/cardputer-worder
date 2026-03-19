//! HAL test: SD card only (SPI3). Pin wiring matches `CardputerHal::new`.

use cardworder::cardputer_hal::sd::cardputer_sd::CardputerSd;
use cardworder::ResultExt;
use esp_idf_hal::delay::FreeRtos;
use esp_idf_svc::hal::peripherals::Peripherals;

fn main() {
    esp_idf_svc::sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    log::info!("hal_test_sd: === start ===");
    log::info!("hal_test_sd: boot — Peripherals::take");
    let peripherals = Peripherals::take().unwrap_or_log("error get peripherals");
    log::info!("hal_test_sd: peripherals ok");

    log::info!("hal_test_sd: CardputerSd::build (SPI3, gpio40 sclk, 39 miso, 14 mosi, 12 cs)");
    let mut sd = CardputerSd::build(
        peripherals.spi3,
        peripherals.pins.gpio40,
        peripherals.pins.gpio39,
        peripherals.pins.gpio14,
        peripherals.pins.gpio12,
    );
    log::info!("hal_test_sd: CardputerSd::build — complete (see sd:* logs above for size)");

    log::info!("hal_test_sd: probe is_file_exists(\"wifi_cfg.jsn\")");
    match sd.is_file_exists("wifi_cfg.jsn") {
        Ok(exists) => log::info!("hal_test_sd: wifi_cfg.jsn exists = {}", exists),
        Err(e) => log::warn!("hal_test_sd: is_file_exists error {:?}", e),
    }

    log::info!("hal_test_sd: entering main loop");
    let mut n = 0u32;
    loop {
        FreeRtos::delay_ms(1000);
        n += 1;
        if n % 5 == 0 {
            log::info!("hal_test_sd: heartbeat {}", n);
        }
    }
}
