use cardworder::cardputer_hal::cardputer_hal::CardputerHal;
use cardworder::cardputer_hal::input::keyboard::{InputLanguage, InputState, PressedSymbol};
use cardworder::cardputer_hal::input::keyboard_io::KeyEvent;
use cardworder::core0_dispatch::{self, Core0Result};
use cardworder::core0_dispatch::executor::HalExecutor;
use cardworder::esp_util;
use cardworder::runtime::Runtime;
use cardworder::screen::Snapshot;
use cardworder::screens::main_menu::MainMenuScreen;
use cardworder::types::{KeyMsg, Msg};
use cardworder::ui::cardworder_ui::{CardworderDisplay, CardworderUi};
use cardworder::ResultExt;
use embedded_graphics::pixelcolor::Rgb565;
use embedded_graphics::prelude::RgbColor;
use esp_idf_hal::delay::FreeRtos;
use esp_idf_hal::task::thread::ThreadSpawnConfiguration;
use esp_idf_svc::eventloop::EspSystemEventLoop;
use esp_idf_svc::hal::peripherals::Peripherals;
use esp_idf_svc::sntp::EspSntp;

fn main() {
    esp_idf_svc::sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    log::info!("Start the app");

    // Init NVS — required by WiFi driver in ESP-IDF 5.5+
    log::info!("boot step 0: NVS init");
    let _nvs = esp_idf_svc::nvs::EspDefaultNvsPartition::take().unwrap_or_log("error init NVS");

    log::info!("boot step 1: Peripherals::take");
    let peripherals = Peripherals::take().unwrap_or_log("error get peripherals");
    log::info!("boot step 2: EspSystemEventLoop::take");
    let sysloop = EspSystemEventLoop::take().unwrap_or_log("error init event loop");
    log::info!("boot step 3: build_all");
    let parts = CardputerHal::build_all(peripherals, sysloop);

    // Keyboard stays on Core 0 main thread
    let mut keyboard = parts.keyboard;

    // Heap-pin display then box as trait object.
    // Safety: Box::leak returns a valid heap pointer; Box::from_raw reclaims it.
    // transmute extends the lifetime to 'static (ESP peripherals live for the entire program).
    let boxed_display: Box<dyn CardworderDisplay> = unsafe {
        let leaked: &'static mut cardworder::cardputer_hal::screen::display::CardputerDisplay<'static> =
            std::mem::transmute(Box::leak(Box::new(parts.display)));
        Box::from_raw(leaked as *mut _)
    };

    // HAL stays on Core 0 (SPI is core-affine)
    let mut hal = parts.hal;
    let mut battery = parts.battery;

    let mut battery_percent: u8 = 0;
    let mut last_battery_read_us: u64 = 0;

    // Migrate old PAIRS.BIN to new two-file format if needed
    match hal.migrate_pairs_if_needed() {
        Ok(true) => log::info!("boot: migrated PAIRS.BIN → FSRS.BIN + WORDS.BIN"),
        Ok(false) => {}
        Err(e) => log::warn!("boot: migration skipped: {:?}", e),
    }

    // Build UI with direct display ownership
    log::info!("boot step 4: CardworderUi::build");
    let mut ui = CardworderUi::build(parts.framebuffer, boxed_display, esp_util::local_time_hms);

    // Splash animation using hardware scroll
    cardworder::ui::splash::run_splash(&mut ui, &mut keyboard);

    // Create channels
    let (msg_tx, msg_rx) = std::sync::mpsc::channel::<Msg>();
    let (state_tx, state_rx) = std::sync::mpsc::channel::<Snapshot>();

    let kb_msg_tx = msg_tx.clone();
    let core0_msg_tx = msg_tx.clone();

    // Spawn Core 1 runtime
    log::info!("boot step 5: spawning runtime on Core 1");
    let runtime_msg_tx = msg_tx;
    let runtime_state_tx = state_tx;

    ThreadSpawnConfiguration {
        name: Some(c"runtime"),
        stack_size: 32768,
        priority: 5,
        pin_to_core: Some(esp_idf_hal::cpu::Core::Core1),
        ..Default::default()
    }
    .set()
    .unwrap();
    std::thread::spawn(move || {
        let runtime = Runtime::new(
            Box::new(MainMenuScreen::default()),
            msg_rx,
            runtime_msg_tx,
            runtime_state_tx,
            |ms| FreeRtos::delay_ms(ms),
            |task| {
                ThreadSpawnConfiguration {
                    pin_to_core: Some(esp_idf_hal::cpu::Core::Core1),
                    stack_size: 8192,
                    priority: 4,
                    name: Some(c"task"),
                    ..Default::default()
                }
                .set()
                .ok();
                std::thread::spawn(task);
            },
        );
        runtime.run();
    });

    // ---- Core 0: keyboard poll + draw loop + Core0Action execution ----
    let heap = esp_util::heap_info();
    log::info!("boot: free heap {}KB / {}KB, entering Core 0 draw loop", heap.free_bytes / 1024, heap.total_bytes / 1024);

    let mut input_state = InputState {
        ctrl_pressed: false,
        shift_pressed: false,
        opt_pressed: false,
        alt_pressed: false,
        fn_pressed: false,
        fn_locked: true,  // Fn locked on by default (arrows work without holding Fn)
        shift_locked: false,
        fn_last_release_us: 0,
        shift_last_release_us: 0,
        lang: InputLanguage::En,
    };
    let mut last_pressed: Option<(KeyEvent, PressedSymbol)> = None;
    let mut current_snapshot: Option<Snapshot> = None;
    let mut last_draw_us: u64 = 0;
    let mut ntp_instance: Option<EspSntp<'static>> = None;
    let mut wifi_connected = false;
    let mut prev_dirty_max_y: usize = 134; // track previous frame's extent

    loop {
        // 1. Poll keyboard
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

        // 3. Draw first (so Loading/Saving screens appear before blocking HAL ops)
        let now_us = esp_util::now_us();
        let timer_tick = (now_us - last_draw_us) >= 1_000_000;
        let should_draw = got_new_snapshot || timer_tick;

        if should_draw {
            if let Some(ref snapshot) = current_snapshot {
                if got_new_snapshot {
                    // Full redraw: clear without dirtying, draw content, mark previous extent
                    ui.clear_no_dirty(Rgb565::BLACK);
                    snapshot.draw(&mut ui);
                    if snapshot.needs_top_line() {
                        ui.draw_top_line(&input_state, &last_pressed, wifi_connected, battery_percent);
                    }
                    // Ensure previously-drawn rows get flushed (to clear old content on display)
                    ui.mark_rows_dirty(0, prev_dirty_max_y);
                } else {
                    // Clock-only redraw: only update top bar (12 rows)
                    if snapshot.needs_top_line() {
                        ui.draw_top_line(&input_state, &last_pressed, wifi_connected, battery_percent);
                    }
                }
                ui.flip_buffer();
                // Remember this frame's dirty extent for next frame
                prev_dirty_max_y = 134; // will be refined once we see actual bbox
            }
            last_draw_us = now_us;
        }

        // 4. Execute Core0Actions AFTER drawing (so Loading/Saving text is visible)
        if got_new_snapshot {
            if let Some(ref mut snapshot) = current_snapshot {
                if let Some(action) = snapshot.action() {
                    let mut executor = HalExecutor { hal: &mut hal, ntp: &mut ntp_instance, battery: &mut battery };
                    let result = core0_dispatch::dispatch(&mut executor, action);
                    match &result {
                        Core0Result::WifiConnected { .. } => wifi_connected = true,
                        Core0Result::WifiStopped => wifi_connected = false,
                        _ => {}
                    }
                    let _ = core0_msg_tx.send(Msg::Core0Result(result));
                }
            }
        }

        // 5. Read battery every 30 seconds
        let now_bat = esp_util::now_us();
        if now_bat - last_battery_read_us >= 30_000_000 || last_battery_read_us == 0 {
            battery_percent = battery.read_percent();
            last_battery_read_us = now_bat;
        }

        // 6. Yield
        FreeRtos::delay_ms(8);
    }
}
