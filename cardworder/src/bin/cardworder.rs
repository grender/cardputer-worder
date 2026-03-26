use std::ffi::CStr;

use cardworder::cardputer_hal::cardputer_hal::CardputerHal;
use cardworder::cardputer_hal::input::keyboard::{InputLanguage, InputState, PressedSymbol};
use cardworder::cardputer_hal::input::keyboard_io::KeyEvent;
use cardworder::runtime::Runtime;
use cardworder::screen::Renderable;
use cardworder::screens::main_menu::MainMenuScreen;
use cardworder::types::{KeyMsg, Msg};
use cardworder::ui::cardworder_ui::CardworderUi;
use cardworder::ResultExt;
use embedded_graphics::pixelcolor::Rgb565;
use embedded_graphics::prelude::RgbColor;
use esp_idf_hal::delay::FreeRtos;
use esp_idf_hal::task::thread::ThreadSpawnConfiguration;
use esp_idf_svc::eventloop::EspSystemEventLoop;
use esp_idf_svc::hal::peripherals::Peripherals;

fn main() {
    esp_idf_svc::sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    log::info!("Start the app");

    log::info!("boot step 1: Peripherals::take");
    let peripherals = Peripherals::take().unwrap_or_log("error get peripherals");
    log::info!("boot step 2: EspSystemEventLoop::take");
    let sysloop = EspSystemEventLoop::take().unwrap_or_log("error init event loop");
    log::info!("boot step 3: build_all");
    let parts = CardputerHal::build_all(peripherals, sysloop);

    // Keyboard stays on Core 0 main thread — no Box::leak needed, just use directly
    let mut keyboard = parts.keyboard;

    // Heap-pin display (ESP-IDF SPI driver has internal registrations that break on move)
    let display: &'static mut _ = unsafe {
        std::mem::transmute::<_, &'static mut cardworder::cardputer_hal::screen::display::CardputerDisplay<'static>>(
            Box::leak(Box::new(parts.display)),
        )
    };

    // Heap-pin HAL (SD SPI driver same issue — must not be moved to thread stack)
    let hal: &'static mut _ = unsafe {
        std::mem::transmute::<_, &'static mut cardworder::cardputer_hal::cardputer_hal::CardputerHal<'static>>(
            Box::leak(Box::new(parts.hal)),
        )
    };

    // Build UI with direct display ownership (no display thread needed)
    log::info!("boot step 4: CardworderUi::build");
    let mut ui = CardworderUi::build(parts.framebuffer, display);

    // Create channels (unbounded — avoids ESP-IDF sync_channel issues)
    let (msg_tx, msg_rx) = std::sync::mpsc::channel::<Msg>();
    let (state_tx, state_rx) = std::sync::mpsc::channel::<Box<dyn Renderable>>();

    let kb_msg_tx = msg_tx.clone();

    // Spawn Core 1 runtime (the ONLY spawned thread)
    log::info!("boot step 5: spawning runtime on Core 1");
    let runtime_msg_tx = msg_tx;
    let runtime_state_tx = state_tx;

    ThreadSpawnConfiguration {
        name: Some(unsafe { CStr::from_bytes_with_nul_unchecked(b"runtime\0") }),
        stack_size: 16384,
        priority: 5,
        pin_to_core: Some(esp_idf_hal::cpu::Core::Core1),
        ..Default::default()
    }
    .set()
    .unwrap();
    std::thread::spawn(move || {
        let runtime = Runtime::new(
            Box::new(MainMenuScreen::default()),
            hal,
            msg_rx,
            runtime_msg_tx,
            runtime_state_tx,
        );
        runtime.run();
    });

    // ---- Core 0: keyboard poll + draw loop (main thread, never blocks) ----
    log::info!("boot: init done, entering Core 0 draw loop");

    let mut input_state = InputState {
        ctrl_pressed: false,
        shift_pressed: false,
        opt_pressed: false,
        alt_pressed: false,
        fn_pressed: false,
        lang: InputLanguage::En,
    };
    let mut last_pressed: Option<(KeyEvent, PressedSymbol)> = None;
    let mut current_snapshot: Option<Box<dyn Renderable>> = None;
    let mut last_draw_us: u64 = 0;

    loop {
        // 1. Poll keyboard (8 GPIO reads — microseconds, never blocks)
        let key = keyboard.read_events();
        if let Some((event, scancode)) = key {
            let pressed = input_state.eat_keys(event, scancode).map(|s| (event, s));
            last_pressed = pressed;
            let key_msg = KeyMsg {
                key: Some((event, scancode)),
                input_state,
                pressed,
            };
            let _ = kb_msg_tx.send(Msg::Key(key_msg));
        }

        // 2. Drain snapshot channel (take latest)
        let mut got_new_snapshot = false;
        while let Ok(snapshot) = state_rx.try_recv() {
            current_snapshot = Some(snapshot);
            got_new_snapshot = true;
        }

        // 3. Draw only when needed (skip-redraw optimization)
        let now_us = unsafe { esp_idf_svc::sys::esp_timer_get_time() as u64 };
        let should_draw = got_new_snapshot || (now_us - last_draw_us) >= 1_000_000;

        if should_draw {
            if let Some(ref snapshot) = current_snapshot {
                ui.clear(Rgb565::BLACK);
                snapshot.draw(&mut ui);
                if snapshot.needs_top_line() {
                    ui.draw_top_line(&input_state, &last_pressed);
                }
                ui.flip_buffer(); // partial SPI flush via dirty tracking
            }
            last_draw_us = now_us;
        }

        // 4. Yield — 8ms matches keyboard poll rate (~125Hz)
        FreeRtos::delay_ms(8);
    }
}
