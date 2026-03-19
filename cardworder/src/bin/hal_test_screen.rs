//! HAL test: display only (SPI2 + ST7789). Pin wiring matches `CardputerHal::new`.

use cardworder::cardputer_hal::screen::cardputer_screen::CardputerScreen;
use cardworder::ResultExt;
use embedded_graphics::{pixelcolor::Rgb565, prelude::{DrawTarget, WebColors}};
use esp_idf_hal::delay::FreeRtos;
use esp_idf_svc::hal::peripherals::Peripherals;

fn main() {
    esp_idf_svc::sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    log::info!("hal_test_screen: === start ===");
    log::info!("hal_test_screen: boot — Peripherals::take");
    let peripherals = Peripherals::take().unwrap_or_log("error get peripherals");
    log::info!("hal_test_screen: peripherals ok");

    log::info!("hal_test_screen: CardputerScreen::build (SPI2, gpio36 sck, 35 dc, 37 cs, 34 rs, 33 rst, 38 bl)");
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
    log::info!("hal_test_screen: CardputerScreen::build — done");

    log::info!("hal_test_screen: backlight on");
    if let Err(e) = screen.backlight_on() {
        log::warn!("hal_test_screen: backlight_on returned {:?}", e);
    }

    log::info!("hal_test_screen: clear framebuffer to RED");
    screen.clear(Rgb565::CSS_RED).unwrap();
    log::info!("hal_test_screen: flush_framebuffer");
    match screen.flush_framebuffer() {
        Ok(()) => log::info!("hal_test_screen: flush_framebuffer — ok"),
        Err(e) => log::error!("hal_test_screen: flush_framebuffer — err {:?}", e),
    }

    log::info!("hal_test_screen: entering main loop (heartbeat every 5s)");
    let mut n = 0u32;
    loop {
        FreeRtos::delay_ms(1000);
        n += 1;
        if n % 5 == 0 {
            log::info!("hal_test_screen: heartbeat tick {}", n);
        }
    }
}
