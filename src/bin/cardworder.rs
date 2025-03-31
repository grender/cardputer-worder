use cardworder::input::{InputLanguage, InputState, PressedSymbol};
use cardworder::keyboard::{CardputerKeyboard, Key, KeyEvent};
use cardworder::screen::{cardputer_screen, cardworder_ui};
use cardworder::sd::cardputer_sd::CardputerSd;
use embedded_fps::{StdClock, FPS};
use embedded_graphics::prelude::WebColors;
use esp_idf_hal::gpio::{self, IOPin, Output, OutputPin, PinDriver};
use esp_idf_svc::hal::prelude::Peripherals;

use embedded_graphics::{pixelcolor::Rgb565, prelude::*};

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

    let peripherals = Peripherals::take().unwrap_or_log("error get peripherals");

    let display = cardputer_screen::CardputerScreen::build(
        Rgb565::CSS_BLACK,
        peripherals.spi2,
        peripherals.pins.gpio36,
        peripherals.pins.gpio35,
        peripherals.pins.gpio37,
        peripherals.pins.gpio34,
        peripherals.pins.gpio33,
        peripherals.pins.gpio38,
    );

    let mut ui = cardworder_ui::CardworderUi::build(display);

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

    /*
    let mut idx: u8 = 0;

    let bg_color = Rgb565::CSS_DARK_GRAY;

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
            "(= Приветики =)",
            display.bounding_box().center() - Point::new(0, 32),
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
    */
}
