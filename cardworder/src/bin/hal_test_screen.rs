//! HAL test: display FPS benchmark. Fills screen with solid color as fast as possible.

use core::fmt::Write;

use cardworder::cardputer_hal::screen::cardputer_screen::CardputerScreen;
use cardworder::ResultExt;
use embedded_fps::FPS;
use embedded_graphics::{
    pixelcolor::Rgb565,
    prelude::{DrawTarget, Point, Size, WebColors},
    primitives::Rectangle,
};
use embedded_graphics::prelude::RgbColor;
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

    log::info!("hal_test_screen: === FPS benchmark start ===");
    let peripherals = Peripherals::take().unwrap_or_log("error get peripherals");

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

    if let Err(e) = screen.backlight_on() {
        log::warn!("hal_test_screen: backlight_on returned {:?}", e);
    }

    let mut fps_counter = FPS::<45, _>::new(HalTestClock {});
    let fps_font = FontRenderer::new::<fonts::u8g2_font_4x6_t_cyrillic>();
    let fps_glyph_size = fps_font.get_glyph_bounding_box(VerticalPosition::Top).size;

    log::info!("hal_test_screen: entering benchmark loop (fill + flush, no delay)");

    let mut n = 0u32;
    let mut frame_sample_count: u64 = 0;
    let mut fill_us_acc: u64 = 0;
    let mut overlay_us_acc: u64 = 0;
    let mut flush_us_acc: u64 = 0;

    loop {
        n = n.wrapping_add(1);

        let fill = match n % 6 {
            0 => Rgb565::CSS_RED,
            1 => Rgb565::CSS_GREEN,
            2 => Rgb565::CSS_BLUE,
            3 => Rgb565::CSS_YELLOW,
            4 => Rgb565::CSS_MAGENTA,
            _ => Rgb565::CSS_CYAN,
        };

        // 1. Fill entire framebuffer
        let t0 = unsafe { esp_idf_svc::sys::esp_timer_get_time() as u64 };
        screen.clear(fill).unwrap();
        let t_after_fill = unsafe { esp_idf_svc::sys::esp_timer_get_time() as u64 };

        // 2. FPS overlay
        let fps = fps_counter.tick();
        let mut fps_text = heapless::String::<16>::new();
        let _ = write!(fps_text, "FPS: {}", fps);
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
        let t_after_overlay = unsafe { esp_idf_svc::sys::esp_timer_get_time() as u64 };

        // 3. Flush to display (no delay — max speed)
        match screen.flush_framebuffer() {
            Ok(()) => {}
            Err(e) => log::error!("hal_test_screen: flush error {:?}", e),
        }
        let t_after_flush = unsafe { esp_idf_svc::sys::esp_timer_get_time() as u64 };

        // Accumulate timing
        fill_us_acc += t_after_fill - t0;
        overlay_us_acc += t_after_overlay - t_after_fill;
        flush_us_acc += t_after_flush - t_after_overlay;
        frame_sample_count += 1;

        if frame_sample_count >= 30 {
            let n = frame_sample_count.max(1);
            log::info!(
                "perf frame avg us (fill={}, overlay={}, flush={}, total={})",
                fill_us_acc / n,
                overlay_us_acc / n,
                flush_us_acc / n,
                (fill_us_acc + overlay_us_acc + flush_us_acc) / n,
            );
            frame_sample_count = 0;
            fill_us_acc = 0;
            overlay_us_acc = 0;
            flush_us_acc = 0;
        }
    }
}
