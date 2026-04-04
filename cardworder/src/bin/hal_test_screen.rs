//! HAL test: font rendering comparison.
//! Shows monospaced vs proportional fonts, English and Russian text.
//! Press any key to cycle through test pages.

use core::fmt::Write;

use cardworder::cardputer_hal::screen::cardputer_screen::CardputerScreen;
use cardworder::cardputer_hal::input::keyboard_io::{CardputerKeyboard, KeyEvent};
use cardworder::ResultExt;
use embedded_graphics::{
    pixelcolor::Rgb565,
    prelude::{DrawTarget, Point, WebColors},
};
use embedded_graphics::prelude::RgbColor;
use esp_idf_hal::delay::FreeRtos;
use esp_idf_hal::gpio::{Output, PinDriver, Pull};
use esp_idf_svc::hal::peripherals::Peripherals;
use u8g2_fonts::types::{FontColor, VerticalPosition};
use u8g2_fonts::{fonts, FontRenderer};

struct FontEntry<'a> {
    name: &'a str,
    renderer: &'a FontRenderer,
    #[allow(dead_code)]
    is_mono: bool,
}

fn main() {
    esp_idf_svc::sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    log::info!("hal_test_screen: === font test start ===");
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
    screen.backlight_on().ok();

    // Build keyboard
    let mux_pins: [PinDriver<'_, Output>; 3] = [
        PinDriver::output(peripherals.pins.gpio8.degrade_output()).unwrap(),
        PinDriver::output(peripherals.pins.gpio9.degrade_output()).unwrap(),
        PinDriver::output(peripherals.pins.gpio11.degrade_output()).unwrap(),
    ];
    let column_pins = [
        PinDriver::input(peripherals.pins.gpio13.degrade_input_output(), Pull::Up).unwrap(),
        PinDriver::input(peripherals.pins.gpio15.degrade_input_output(), Pull::Up).unwrap(),
        PinDriver::input(peripherals.pins.gpio3.degrade_input_output(), Pull::Up).unwrap(),
        PinDriver::input(peripherals.pins.gpio4.degrade_input_output(), Pull::Up).unwrap(),
        PinDriver::input(peripherals.pins.gpio5.degrade_input_output(), Pull::Up).unwrap(),
        PinDriver::input(peripherals.pins.gpio6.degrade_input_output(), Pull::Up).unwrap(),
        PinDriver::input(peripherals.pins.gpio7.degrade_input_output(), Pull::Up).unwrap(),
    ];
    let mut keyboard = CardputerKeyboard::new(mux_pins, column_pins);
    keyboard.init();

    // --- Monospaced fonts (NxM pattern, fixed-width grid) ---
    let mono_4x6 = FontRenderer::new::<fonts::u8g2_font_4x6_t_cyrillic>();
    let mono_5x8 = FontRenderer::new::<fonts::u8g2_font_5x8_t_cyrillic>();
    let mono_6x12 = FontRenderer::new::<fonts::u8g2_font_6x12_t_cyrillic>();
    let mono_9x15 = FontRenderer::new::<fonts::u8g2_font_9x15_t_cyrillic>();
    let mono_10x20 = FontRenderer::new::<fonts::u8g2_font_10x20_t_cyrillic>();
    let mono_7x13 = FontRenderer::new::<fonts::u8g2_font_7x13_t_cyrillic>();
    let mono_8x13 = FontRenderer::new::<fonts::u8g2_font_8x13_t_cyrillic>();

    // --- Proportional fonts (variable-width glyphs) ---
    let prop_cu12 = FontRenderer::new::<fonts::u8g2_font_cu12_t_cyrillic>();
    let prop_inr24 = FontRenderer::new::<fonts::u8g2_font_inr24_t_cyrillic>();
    let prop_inr27 = FontRenderer::new::<fonts::u8g2_font_inr27_t_cyrillic>();
    let prop_haxr = FontRenderer::new::<fonts::u8g2_font_haxrcorp4089_t_cyrillic>();
    let prop_unifont = FontRenderer::new::<fonts::u8g2_font_unifont_t_cyrillic>();

    let en_text = "Hello World! iiiWWW";
    let ru_text = "Привет Мир! шшшІІІ";
    let _mix_text = "Mix: Слово - word";

    let all_fonts: Vec<FontEntry> = vec![
        // Page 1: Monospaced small
        FontEntry { name: "4x6 mono", renderer: &mono_4x6, is_mono: true },
        FontEntry { name: "5x8 mono", renderer: &mono_5x8, is_mono: true },
        FontEntry { name: "6x12 mono", renderer: &mono_6x12, is_mono: true },
        FontEntry { name: "7x13 mono", renderer: &mono_7x13, is_mono: true },
        FontEntry { name: "8x13 mono", renderer: &mono_8x13, is_mono: true },
        // Page 2: Monospaced large
        FontEntry { name: "9x15 mono", renderer: &mono_9x15, is_mono: true },
        FontEntry { name: "10x20 mono", renderer: &mono_10x20, is_mono: true },
        // Page 3: Proportional
        FontEntry { name: "cu12 prop", renderer: &prop_cu12, is_mono: false },
        FontEntry { name: "haxrcorp prop", renderer: &prop_haxr, is_mono: false },
        FontEntry { name: "unifont prop", renderer: &prop_unifont, is_mono: false },
        // Page 4: Proportional large
        FontEntry { name: "inr24 prop", renderer: &prop_inr24, is_mono: false },
        FontEntry { name: "inr27 prop", renderer: &prop_inr27, is_mono: false },
    ];

    let label_font = &mono_4x6;
    let mut page = 0usize;
    let mut needs_redraw = true;

    // Group fonts into pages that fit on screen
    let pages: Vec<Vec<usize>> = build_pages(&all_fonts, &screen);

    loop {
        if needs_redraw {
            screen.clear(Rgb565::BLACK).unwrap();

            let page_fonts = &pages[page % pages.len()];
            let mut y = 2i32;

            // Page header
            let mut header = heapless::String::<32>::new();
            let _ = write!(header, "Page {}/{}", page % pages.len() + 1, pages.len());
            label_font.render(
                header.as_str(), Point::new(0, y), VerticalPosition::Top,
                FontColor::Transparent(Rgb565::CSS_GRAY), &mut screen,
            ).ok();
            y += 8;

            for &fi in page_fonts {
                let entry = &all_fonts[fi];
                let line_h = entry.renderer.get_default_line_height() as i32;

                // Font label (small gray)
                let mut label = heapless::String::<32>::new();
                let _ = write!(label, "{}", entry.name);
                label_font.render(
                    label.as_str(), Point::new(0, y), VerticalPosition::Top,
                    FontColor::Transparent(Rgb565::CSS_DARK_GRAY), &mut screen,
                ).ok();
                y += 7;

                // English text
                entry.renderer.render(
                    en_text, Point::new(0, y), VerticalPosition::Top,
                    FontColor::Transparent(Rgb565::WHITE), &mut screen,
                ).ok();
                y += line_h + 1;

                // Russian text
                entry.renderer.render(
                    ru_text, Point::new(0, y), VerticalPosition::Top,
                    FontColor::Transparent(Rgb565::CSS_LIGHT_BLUE), &mut screen,
                ).ok();
                y += line_h + 1;

                // Separator
                y += 2;
            }

            // Hint at bottom
            label_font.render(
                "Any key = next page", Point::new(0, 128), VerticalPosition::Top,
                FontColor::Transparent(Rgb565::CSS_GRAY), &mut screen,
            ).ok();

            screen.flush_framebuffer().ok();
            needs_redraw = false;
        }

        if let Some((KeyEvent::Pressed, _)) = keyboard.read_events() {
            page += 1;
            needs_redraw = true;
        }

        FreeRtos::delay_ms(10);
    }
}

/// Split font indices into pages that fit within 135px screen height.
fn build_pages<'a>(fonts: &[FontEntry<'a>], _screen: &CardputerScreen) -> Vec<Vec<usize>> {
    let mut pages: Vec<Vec<usize>> = Vec::new();
    let mut current_page: Vec<usize> = Vec::new();
    let mut y = 10i32; // header
    let max_y = 125; // leave room for hint

    for (i, entry) in fonts.iter().enumerate() {
        let line_h = entry.renderer.get_default_line_height() as i32;
        let block_h = 7 + line_h * 2 + 2 + 4; // label + 2 text lines + gaps

        if y + block_h > max_y && !current_page.is_empty() {
            pages.push(current_page);
            current_page = Vec::new();
            y = 10;
        }
        current_page.push(i);
        y += block_h;
    }
    if !current_page.is_empty() {
        pages.push(current_page);
    }
    pages
}
