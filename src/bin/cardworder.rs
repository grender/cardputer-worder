use cardworder::keyboard::{Keyboard, KeyboardState};
use cardworder::screen::cardputer_screen;
use cardworder::screen::display::{DISPLAY_SIZE_HEIGHT, DISPLAY_SIZE_WIDTH};
use embedded_fps::{StdClock, FPS};
use embedded_graphics::mono_font::ascii::{FONT_4X6, FONT_9X18_BOLD};
use embedded_graphics::prelude::WebColors;
use embedded_graphics::primitives::Line;
use esp_idf_svc::hal::prelude::Peripherals;

use embedded_graphics::{pixelcolor::Rgb565, prelude::*, primitives::PrimitiveStyle};

use embedded_graphics::mono_font::MonoTextStyle;
use embedded_graphics::text::Text;
use u8g2_fonts::types::{FontColor, HorizontalAlignment, VerticalPosition};
use u8g2_fonts::{fonts, FontRenderer};

pub trait ResultExt<R, E> {
    fn unwrap_or_log(self, message: &str) -> R;
}

impl<R, E: std::fmt::Debug> ResultExt<R, E> for Result<R, E> {
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

    let std_clock = StdClock::default();
    let mut fps_counter = FPS::<120, _>::new(std_clock);

    let peripherals = Peripherals::take().unwrap_or_log("error get peripherals");

    let mut display = cardputer_screen::CardputerScreen::build(
        Rgb565::CSS_BLACK,
        peripherals.spi2,
        peripherals.pins.gpio36,
        peripherals.pins.gpio35,
        peripherals.pins.gpio37,
        peripherals.pins.gpio34,
        peripherals.pins.gpio33,
        peripherals.pins.gpio38,
    );

    let mut idx: u8 = 0;

    let bg_color = Rgb565::CSS_LIGHT_GRAY;

    let font = FontRenderer::new::<fonts::u8g2_font_haxrcorp4089_t_cyrillic>();

    let mut keyboard = Keyboard::new(
        peripherals.pins.gpio8,
        peripherals.pins.gpio9,
        peripherals.pins.gpio11,
        peripherals.pins.gpio13,
        peripherals.pins.gpio15,
        peripherals.pins.gpio3,
        peripherals.pins.gpio4,
        peripherals.pins.gpio5,
        peripherals.pins.gpio6,
        peripherals.pins.gpio7,
    )
    .unwrap();
    let mut keyboard_state = KeyboardState::default();

    let mut add = 0;

    loop {
        keyboard_state.update(&mut keyboard).unwrap();
        log::info!("pressed : {:?}", keyboard_state.pressed_keys());
        log::info!("released: {:?}", keyboard_state.released_keys());

        // thread::sleep(Duration::from_millis(50));
        (idx, _) = idx.overflowing_add(add);
        add = 0;

        //display.screen.set_vertical_scroll_region(20, DISPLAY_SIZE_HEIGHT); // .unwrap();
        //display.screen.set_vertical_scroll_offset(idx.into()); // .unwrap();

        let text_color = Rgb565::new(idx, 0b1111_1111 ^ idx, idx);
        let text_small_color = Rgb565::new(0b1111_1111 ^ idx, idx, idx);

        let style = MonoTextStyle::new(&FONT_9X18_BOLD, text_color);
        let style_small = MonoTextStyle::new(&FONT_4X6, text_small_color);

        // log::info!("clear");
        display
            .clear(bg_color)
            .map_err(|err| {
                log::error!("error clear {:?}", err);
            })
            .unwrap_or_log("error clear display");

        font.render_aligned(
            "Приветики =)",
            display.bounding_box().center() + Point::new(0, 16),
            VerticalPosition::Baseline,
            HorizontalAlignment::Center,
            FontColor::Transparent(Rgb565::RED),
            &mut display,
        )
        .unwrap_or_log("error draw with nice font");

        // cross
        // log::info!("cdraw cross");
        Line::new(
            Point::new(0, 0),
            Point::new(
                (DISPLAY_SIZE_WIDTH - 1).into(),
                (DISPLAY_SIZE_HEIGHT - 1).into(),
            ),
        )
        .into_styled(PrimitiveStyle::with_stroke(text_color, 1))
        .draw(&mut display)
        .unwrap_or_log("error draw line");

        Line::new(
            Point::new(0, (DISPLAY_SIZE_HEIGHT - 1).into()),
            Point::new((DISPLAY_SIZE_WIDTH - 1).into(), 0),
        )
        .into_styled(PrimitiveStyle::with_stroke(text_color, 1))
        .draw(&mut display)
        .unwrap_or_log("error draw line");

        // perimeter
        // log::info!("cdraw perimiter");
        Line::new(
            Point::new(0, 0),
            Point::new((DISPLAY_SIZE_WIDTH - 1).into(), 0),
        )
        .into_styled(PrimitiveStyle::with_stroke(Rgb565::CSS_RED, 1))
        .draw(&mut display)
        .unwrap_or_log("error draw line");

        Line::new(
            Point::new(0, 0),
            Point::new(0, (DISPLAY_SIZE_HEIGHT - 1).into()),
        )
        .into_styled(PrimitiveStyle::with_stroke(Rgb565::CSS_RED, 1))
        .draw(&mut display)
        .unwrap_or_log("error draw line");

        Line::new(
            Point::new(
                (DISPLAY_SIZE_WIDTH - 1).into(),
                (DISPLAY_SIZE_HEIGHT - 1).into(),
            ),
            Point::new((DISPLAY_SIZE_WIDTH - 1).into(), 0),
        )
        .into_styled(PrimitiveStyle::with_stroke(Rgb565::CSS_RED, 1))
        .draw(&mut display)
        .unwrap_or_log("error draw line");

        Line::new(
            Point::new(
                (DISPLAY_SIZE_WIDTH - 1).into(),
                (DISPLAY_SIZE_HEIGHT - 1).into(),
            ),
            Point::new(0, (DISPLAY_SIZE_HEIGHT - 1).into()),
        )
        .into_styled(PrimitiveStyle::with_stroke(Rgb565::CSS_RED, 1))
        .draw(&mut display)
        .unwrap_or_log("error draw line");

        // log::info!("cdraw coords");
        let p = Point::new(0, 0);
        let s = format!("{} , {}", p.x, p.y);
        let mut t = Text::new(&s, p, style_small);
        t.position.y = t.position.y + t.bounding_box().size.height as i32;
        t.draw(&mut display).unwrap_or_log("error draw point text");

        let p = Point::new(0, (DISPLAY_SIZE_HEIGHT - 1).into());
        let s = format!("{} , {}", p.x, p.y);
        let mut t = Text::new(&s, p, style_small);
        t.position.y = t.position.y - t.bounding_box().size.height as i32;
        t.draw(&mut display).unwrap_or_log("error draw point text");

        let p = Point::new(
            (DISPLAY_SIZE_WIDTH - 1).into(),
            (DISPLAY_SIZE_HEIGHT - 1).into(),
        );
        let s = format!("{} , {}", p.x, p.y);
        let mut t = Text::new(&s, p, style_small);
        t.position.x = t.position.x - t.bounding_box().size.width as i32;
        t.position.y = t.position.y - t.bounding_box().size.height as i32;
        t.draw(&mut display).unwrap_or_log("error draw point text");

        let p = Point::new((DISPLAY_SIZE_WIDTH - 1).into(), 0);
        let s = format!("{} , {}", p.x, p.y);
        let mut t = Text::new(&s, p, style_small);
        t.position.x = t.position.x - t.bounding_box().size.width as i32;
        t.position.y = t.position.y + t.bounding_box().size.height as i32;
        t.draw(&mut display).unwrap_or_log("error draw point text");

        // log::info!("calc and draw cross");

        let fps = fps_counter.tick_max();

        Text::new(&format!("{} FPS: {}", idx, fps), Point::new(100, 80), style)
            .draw(&mut display)
            .unwrap_or_log("error draw fps text");

        display.flush_framebuffer().unwrap_or_log("error flushing buffer");
    }
}
