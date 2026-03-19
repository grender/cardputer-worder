//! HAL test: WiFi driver init only (modem + `EspSystemEventLoop`). No association.

use cardworder::cardputer_hal::wifi::wifi::CardWorderWifi;
use cardworder::ResultExt;
use esp_idf_hal::delay::FreeRtos;
use esp_idf_svc::eventloop::EspSystemEventLoop;
use esp_idf_svc::hal::peripherals::Peripherals;
use esp_idf_svc::wifi::EspWifi;

fn main() {
    esp_idf_svc::sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    log::info!("hal_test_wifi: === start ===");
    log::info!("hal_test_wifi: boot — Peripherals::take");
    let peripherals = Peripherals::take().unwrap_or_log("error get peripherals");
    log::info!("hal_test_wifi: peripherals ok");

    log::info!("hal_test_wifi: EspSystemEventLoop::take");
    let sysloop = EspSystemEventLoop::take().unwrap_or_log("error init event loop");
    log::info!("hal_test_wifi: event loop ok");

    log::info!("hal_test_wifi: EspWifi::new(modem, sysloop, None)");
    let esp_wifi = EspWifi::new(peripherals.modem, sysloop, None).unwrap_or_log("EspWifi::new failed");
    log::info!("hal_test_wifi: EspWifi::new — done");

    log::info!("hal_test_wifi: CardWorderWifi::new (wrap driver)");
    let _wifi = CardWorderWifi::new(esp_wifi);
    log::info!("hal_test_wifi: WiFi driver initialized (no connect); entering heartbeat loop");

    let mut n = 0u32;
    loop {
        FreeRtos::delay_ms(1000);
        n += 1;
        if n % 5 == 0 {
            log::info!("hal_test_wifi: heartbeat {}", n);
        }
    }
}
