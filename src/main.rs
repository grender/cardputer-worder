pub mod lib;

use crate::lib::display;
use display_interface::DataFormat;
use embedded_fps::{StdClock, FPS};
use embedded_graphics::framebuffer::{buffer_size, Framebuffer};
use embedded_graphics::mono_font::ascii::{FONT_4X6, FONT_9X18_BOLD};
use embedded_graphics::prelude::WebColors;
use embedded_graphics::primitives::Line;
use esp_idf_hal::delay::Delay;
use esp_idf_svc::hal::prelude::Peripherals;

use embedded_graphics::{
    pixelcolor::{raw::LittleEndian, Rgb565},
    prelude::*,
    primitives::PrimitiveStyle,
};

use display_interface::WriteOnlyDataCommand;
use embedded_graphics::mono_font::MonoTextStyle;
use embedded_graphics::text::Text;
use lib::display::{DISPLAY_SIZE_HEIGHT, DISPLAY_SIZE_WIDTH};
use mipidsi::dcs::WriteMemoryStart;

//const DISPLAY_SIZE_WIDTH_U: usize = DISPLAY_SIZE_WIDTH as usize;
//const DISPLAY_SIZE_HEIGHT_U: usize = DISPLAY_SIZE_HEIGHT as usize;

fn main() {
    // It is necessary to call this function once. Otherwise some patches to the runtime
    // implemented by esp-idf-sys might not link properly. See https://github.com/esp-rs/esp-idf-template/issues/71
    esp_idf_svc::sys::link_patches();

    // Bind the log crate to the ESP Logging facilities
    esp_idf_svc::log::EspLogger::initialize_default();

    log::info!("Start the app");

    // let std_clock = StdClock::default();
    // let mut fps_counter = FPS::<120, _>::new(std_clock);

    let peripherals = Peripherals::take().unwrap();

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

    let mut idx: u8 = 0;

    let bg_color = Rgb565::CSS_LIGHT_GRAY;

    let mut display = Framebuffer::<Rgb565, _, LittleEndian, 320, 320, {buffer_size::<Rgb565>(320, 320)}>::new();
    // let mut display = display_real.screen;

    loop {
        (idx, _) = idx.overflowing_add(1);

        //display.screen.set_vertical_scroll_region(20, DISPLAY_SIZE_HEIGHT).unwrap();
        //display.screen.set_vertical_scroll_offset(idx.into()).unwrap();

        let text_color = Rgb565::new(idx, 0b1111_1111 ^ idx, idx);
        let text_small_color = Rgb565::new(0b1111_1111 ^ idx, idx, idx);

        let style = MonoTextStyle::new(&FONT_9X18_BOLD, text_color);
        let style_small = MonoTextStyle::new(&FONT_4X6, text_small_color);

        log::info!("clear");
        display
            .clear(bg_color)
            .map_err(|err| {
                log::error!("error clear {:?}", err);
            })
            .unwrap();

        // cross
        log::info!("cdraw cross");
        Line::new(
            Point::new(0, 0),
            Point::new(
                (DISPLAY_SIZE_WIDTH - 1).into(),
                (DISPLAY_SIZE_HEIGHT - 1).into(),
            ),
        )
        .into_styled(PrimitiveStyle::with_stroke(text_color, 1))
        .draw(&mut display)
        .unwrap();

        Line::new(
            Point::new(0, (DISPLAY_SIZE_HEIGHT - 1).into()),
            Point::new((DISPLAY_SIZE_WIDTH - 1).into(), 0),
        )
        .into_styled(PrimitiveStyle::with_stroke(text_color, 1))
        .draw(&mut display)
        .unwrap();

        // perimeter
        log::info!("cdraw perimiter");
        Line::new(
            Point::new(0, 0),
            Point::new((DISPLAY_SIZE_WIDTH - 1).into(), 0),
        )
        .into_styled(PrimitiveStyle::with_stroke(Rgb565::CSS_RED, 1))
        .draw(&mut display)
        .unwrap();

        Line::new(
            Point::new(0, 0),
            Point::new(0, (DISPLAY_SIZE_HEIGHT - 1).into()),
        )
        .into_styled(PrimitiveStyle::with_stroke(Rgb565::CSS_RED, 1))
        .draw(&mut display)
        .unwrap();

        Line::new(
            Point::new(
                (DISPLAY_SIZE_WIDTH - 1).into(),
                (DISPLAY_SIZE_HEIGHT - 1).into(),
            ),
            Point::new((DISPLAY_SIZE_WIDTH - 1).into(), 0),
        )
        .into_styled(PrimitiveStyle::with_stroke(Rgb565::CSS_RED, 1))
        .draw(&mut display)
        .unwrap();

        Line::new(
            Point::new(
                (DISPLAY_SIZE_WIDTH - 1).into(),
                (DISPLAY_SIZE_HEIGHT - 1).into(),
            ),
            Point::new(0, (DISPLAY_SIZE_HEIGHT - 1).into()),
        )
        .into_styled(PrimitiveStyle::with_stroke(Rgb565::CSS_RED, 1))
        .draw(&mut display)
        .unwrap();

        log::info!("cdraw coords");
        let p = Point::new(0, 0);
        let s = format!("{} , {}", p.x, p.y);
        let mut t = Text::new(&s, p, style_small);
        t.position.y = t.position.y + t.bounding_box().size.height as i32;
        t.draw(&mut display).unwrap();

        let p = Point::new(0, (DISPLAY_SIZE_HEIGHT - 1).into());
        let s = format!("{} , {}", p.x, p.y);
        let mut t = Text::new(&s, p, style_small);
        t.position.y = t.position.y - t.bounding_box().size.height as i32;
        t.draw(&mut display).unwrap();

        let p = Point::new(
            (DISPLAY_SIZE_WIDTH - 1).into(),
            (DISPLAY_SIZE_HEIGHT - 1).into(),
        );
        let s = format!("{} , {}", p.x, p.y);
        let mut t = Text::new(&s, p, style_small);
        t.position.x = t.position.x - t.bounding_box().size.width as i32;
        t.position.y = t.position.y - t.bounding_box().size.height as i32;
        t.draw(&mut display).unwrap();

        let p = Point::new((DISPLAY_SIZE_WIDTH - 1).into(), 0);
        let s = format!("{} , {}", p.x, p.y);
        let mut t = Text::new(&s, p, style_small);
        t.position.x = t.position.x - t.bounding_box().size.width as i32;
        t.position.y = t.position.y + t.bounding_box().size.height as i32;
        t.draw(&mut display).unwrap();

        log::info!("calc and draw cross");

        //let fps = fps_counter.tick_max();

        //Text::new(&format!("{} FPS: {}", idx, fps), Point::new(100, 60), style)
        //    .draw(&mut display)
        //    .unwrap();

        unsafe {
            display_real
                .screen
                .dcs()
                .write_command(WriteMemoryStart)
                .unwrap();
            let buf = DataFormat::U8(display.data_mut());
            display_real.screen.dcs().di.send_data(buf).unwrap();
        }
    }
}
