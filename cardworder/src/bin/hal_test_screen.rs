//! HAL test: display only (SPI2 + ST7789). Pin wiring matches `CardputerHal::new`.

use cardworder::cardputer_hal::screen::cardputer_screen::CardputerScreen;
use cardworder::ResultExt;
use embedded_fps::FPS;
use embedded_graphics::{
    pixelcolor::Rgb565,
    prelude::{DrawTarget, Point, Size, WebColors},
    primitives::Rectangle,
};
use embedded_graphics::prelude::RgbColor;
use esp_idf_hal::delay::FreeRtos;
use esp_idf_svc::hal::peripherals::Peripherals;
use embedded_time::rate::Fraction;
use u8g2_fonts::types::{FontColor, VerticalPosition};
use u8g2_fonts::{fonts, FontRenderer};

struct HalTestClock {}

impl embedded_time::clock::Clock for HalTestClock {
    type T = u64;
    const SCALING_FACTOR: Fraction = Fraction::new(1, 1_000_000);

    fn try_now(&self) -> Result<embedded_time::Instant<Self>, embedded_time::clock::Error> {
        let now = unsafe { esp_idf_svc::sys::esp_timer_get_time() };
        Ok(embedded_time::Instant::<Self>::new(now as u64))
    }
}

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

    // FPS + top-left overlay, similar to `CardworderUi::flip_buffer()`.
    let mut fps_counter = FPS::<45, _>::new(HalTestClock {});
    let fps_font = FontRenderer::new::<fonts::u8g2_font_4x6_t_cyrillic>();
    let fps_glyph_size = fps_font.get_glyph_bounding_box(VerticalPosition::Top).size;

    log::info!("hal_test_screen: entering main loop (color fill + FPS overlay)");
    let mut n = 0u32;
    loop {
        // Run fast enough to make FPS readable.
        FreeRtos::delay_ms(16);
        n = n.wrapping_add(1);

        // Fill whole framebuffer each iteration (acts as a visual "heartbeat").
        let fill = match n % 6 {
            0 => Rgb565::CSS_RED,
            1 => Rgb565::CSS_GREEN,
            2 => Rgb565::CSS_BLUE,
            3 => Rgb565::CSS_YELLOW,
            4 => Rgb565::CSS_MAGENTA,
            _ => Rgb565::CSS_CYAN,
        };
      //  screen.clear(fill).unwrap();

        // Draw FPS label on top-left.
        let fps = fps_counter.tick();
        let fps_text = format!("FPS: {}", fps);
        let area = Rectangle {
            top_left: Point::new(0, 0),
            size: Size::new(
                fps_glyph_size.width as u32 * fps_text.len() as u32 + 2,
                fps_glyph_size.height as u32 + 2,
            ),
        };
        let _ = screen.fill_solid(&area, Rgb565::BLACK);
        fps_font
            .render(
                fps_text.as_str(),
                Point::new(1, 1),
                VerticalPosition::Top,
                FontColor::Transparent(Rgb565::WHITE),
                &mut screen,
            )
            .unwrap();

        // Push framebuffer to the display.
        
        match screen.flush_framebuffer() {
            Ok(()) => {}
            Err(e) => log::error!("hal_test_screen: flush_framebuffer — err {:?}", e),
        }
    }
}
