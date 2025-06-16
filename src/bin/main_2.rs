use cardworder::display;
use display_interface::DataFormat;
use embedded_graphics::{mono_font::MonoTextStyle, pixelcolor::Rgb565, prelude::WebColors};
use embedded_graphics_framebuf::FrameBuf;
use esp_idf_hal::delay::Delay;
use esp_idf_hal::prelude::Peripherals;
use esp_idf_sys::*;
use smart_leds::hsv::{hsv2rgb, Hsv};
use smart_leds_trait::SmartLedsWrite;
use ws2812_esp32_rmt_driver::Ws2812Esp32Rmt;

use embedded_graphics::mono_font::ascii::FONT_9X18_BOLD;

use embedded_graphics::prelude::*;

use display_interface::WriteOnlyDataCommand;
use embedded_graphics::text::Text;
// use crate::display::{DISPLAY_SIZE_HEIGHT, DISPLAY_SIZE_WIDTH};
use mipidsi::dcs::WriteMemoryStart;

fn main() {
    // It is necessary to call this function once. Otherwise some patches to the runtime
    // implemented by esp-idf-sys might not link properly. See https://github.com/esp-rs/esp-idf-template/issues/71
    esp_idf_svc::sys::link_patches();

    // Bind the log crate to the ESP Logging facilities
    esp_idf_svc::log::EspLogger::initialize_default();

    log::info!("Start the app");

    let mut delay = Delay::new_default();
    let peripherals = Peripherals::take().unwrap();
    let channel = peripherals.rmt.channel3;
    let mut ws2812 = Ws2812Esp32Rmt::new(channel, peripherals.pins.gpio21).unwrap();

    let mut hue = unsafe { esp_random() } as u8;

    let mut framebuffer_data = [Rgb565::CSS_LIGHT_GRAY; 240 * 135];
    let mut display = FrameBuf::new(&mut framebuffer_data, 240, 135);

    let mut display_real = display::build(
        peripherals.spi2,
        peripherals.pins.gpio36,
        peripherals.pins.gpio35,
        peripherals.pins.gpio37,
        peripherals.pins.gpio34,
        peripherals.pins.gpio33,
        peripherals.pins.gpio38,
    )
    .unwrap();

    loop {
        let pixels = std::iter::repeat(hsv2rgb(Hsv {
            hue,
            sat: 255,
            val: 8,
        }))
        .take(1);
        ws2812.write(pixels);

        delay.delay_ms(100);

        hue = hue.wrapping_add(10);

        //        display
        //            .clear(Rgb565::CSS_DARK_GRAY)
        //            .unwrap();
        //        let style = MonoTextStyle::new(&FONT_9X18_BOLD, Rgb565::CSS_ALICE_BLUE);
        //        Text::new("=)", Point::new(100, 60), style)
        //        .draw(&mut display)
        //        .unwrap();

        //unsafe {
        //    display_real
        //        .screen
        //        .dcs()
        //        .write_command(WriteMemoryStart)
        //        .unwrap();
        //
        //                //let buf = DataFormat::U8(framebuffer_data);
        //                let mut iter = display.data.into_iter().map(|c| c.into_storage());
        //                let buf = DataFormat::U16BEIter(&mut iter);
        //
        //    display_real.screen.dcs().di.send_data(buf).unwrap();
        //}
    }
}
