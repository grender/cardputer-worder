use std::ffi::CStr;

use cardworder::cardputer_hal::cardputer_hal::CardputerHal;
use cardworder::cardputer_hal::input::keyboard::{InputLanguage, InputState, PressedSymbol};
use cardworder::cardputer_hal::input::keyboard_io::KeyEvent;
use cardworder::runtime::Runtime;
use cardworder::screen::Snapshot;
use cardworder::screens::main_menu::MainMenuScreen;
use cardworder::types::{Core0Action, Core0Result, KeyMsg, Msg};
use cardworder::ui::cardworder_ui::CardworderUi;
use cardworder::ResultExt;
use embedded_graphics::pixelcolor::Rgb565;
use embedded_graphics::prelude::RgbColor;
use esp_idf_hal::delay::FreeRtos;
use esp_idf_hal::task::thread::ThreadSpawnConfiguration;
use esp_idf_svc::eventloop::EspSystemEventLoop;
use esp_idf_svc::hal::peripherals::Peripherals;
use esp_idf_svc::sntp::{EspSntp, SyncStatus};

/// Execute a Core0Action using the HAL on Core 0.
fn execute_core0_action(
    hal: &mut CardputerHal<'_>,
    action: Core0Action,
    ntp: &mut Option<EspSntp<'static>>,
) -> Core0Result {
    match action {
        Core0Action::SetTimezone => {
            unsafe {
                let env_tz = b"TZ\0";
                let tz = b"GMT-3\0";
                esp_idf_sys::setenv(env_tz.as_ptr() as *const u8, tz.as_ptr() as *const u8, 1);
                esp_idf_sys::tzset();
            }
            Core0Result::TimezoneSet
        }
        Core0Action::CreateWifiFileIfNotExists { ssid, password } => {
            match hal.create_wifi_file_if_non_exists(ssid, password) {
                Ok(()) => Core0Result::WifiFileCreated,
                Err(e) => Core0Result::Error(format!("Create wifi file: {:?}", e)),
            }
        }
        Core0Action::LoadWifiConfig => {
            match hal.load_wifi_config() {
                Ok(config) => Core0Result::WifiConfigLoaded(config),
                Err(e) => Core0Result::Error(format!("Load config: {:?}", e)),
            }
        }
        Core0Action::ConnectWifi(config) => {
            let ssid = config.ssid.clone();
            match hal.connect_wifi(config) {
                Ok(()) => Core0Result::WifiConnected { ssid },
                Err(e) => Core0Result::Error(format!("Connect WiFi: {:?}", e)),
            }
        }
        Core0Action::StartNtp => {
            match EspSntp::new_default() {
                Ok(sntp) => {
                    *ntp = Some(sntp);
                    Core0Result::NtpStarted
                }
                Err(e) => Core0Result::Error(format!("Start NTP: {:?}", e)),
            }
        }
        Core0Action::CheckNtpStatus => {
            if let Some(ref sntp) = ntp {
                let done = sntp.get_sync_status() == SyncStatus::Completed;
                if done {
                    *ntp = None; // drop the SNTP instance
                }
                Core0Result::NtpSynced(done)
            } else {
                Core0Result::NtpSynced(true) // no NTP instance, consider done
            }
        }
        Core0Action::StopWifi => {
            match hal.stop_wifi() {
                Ok(()) => Core0Result::WifiStopped,
                Err(e) => Core0Result::Error(format!("Stop WiFi: {:?}", e)),
            }
        }
        Core0Action::LoadPairs => {
            match hal.load_pairs() {
                Ok(file) => Core0Result::PairsLoaded(file),
                Err(e) => Core0Result::Error(format!("Load pairs: {:?}", e)),
            }
        }
        Core0Action::SavePairsBytes(bytes) => {
            match hal.save_pairs_bytes(&bytes) {
                Ok(()) => Core0Result::PairsSaved,
                Err(e) => Core0Result::Error(format!("Save pairs: {:?}", e)),
            }
        }
        Core0Action::LoadWifiList => {
            match hal.load_wifi_list() {
                Ok(list) => Core0Result::WifiListLoaded(list),
                Err(e) => Core0Result::Error(format!("Load WiFi list: {:?}", e)),
            }
        }
        Core0Action::SaveWifiList(list) => {
            match hal.save_wifi_list(&list) {
                Ok(()) => Core0Result::WifiListSaved,
                Err(e) => Core0Result::Error(format!("Save WiFi list: {:?}", e)),
            }
        }
        Core0Action::StartWifi => {
            match hal.start_wifi() {
                Ok(()) => Core0Result::WifiStarted,
                Err(e) => Core0Result::Error(format!("Start WiFi: {:?}", e)),
            }
        }
        Core0Action::StartWifiScan => {
            // scan() is blocking — starts scan and waits for results
            match hal.scan_wifi() {
                Ok(results) => {
                    let networks: Vec<cardworder::types::ScannedNetwork> = results.iter().map(|ap| {
                        cardworder::types::ScannedNetwork {
                            ssid: ap.ssid.clone(),
                            signal_strength: ap.signal_strength,
                            has_saved_password: false, // will be set by screen
                        }
                    }).collect();
                    Core0Result::WifiScanResults(networks)
                }
                Err(e) => Core0Result::Error(format!("WiFi scan: {:?}", e)),
            }
        }
        Core0Action::GetScanResults => {
            Core0Result::Error("use StartWifiScan instead".to_string())
        }
        Core0Action::GetNetworkInfo => {
            let mut ip_str = heapless::String::<16>::new();
            unsafe {
                let netif = esp_idf_sys::esp_netif_get_handle_from_ifkey(
                    b"WIFI_STA_DEF\0".as_ptr() as *const _,
                );
                if !netif.is_null() {
                    let mut ip_info: esp_idf_sys::esp_netif_ip_info_t = core::mem::zeroed();
                    if esp_idf_sys::esp_netif_get_ip_info(netif, &mut ip_info) == 0 {
                        let ip = ip_info.ip.addr;
                        if ip != 0 {
                            let _ = core::fmt::Write::write_fmt(
                                &mut ip_str,
                                format_args!(
                                    "{}.{}.{}.{}",
                                    ip & 0xFF,
                                    (ip >> 8) & 0xFF,
                                    (ip >> 16) & 0xFF,
                                    (ip >> 24) & 0xFF
                                ),
                            );
                        }
                    }
                }
            }
            Core0Result::NetworkInfo { ip: ip_str }
        }
    }
}

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

    // Heap-pin display (ESP-IDF SPI driver has internal registrations that break on move)
    let display: &'static mut _ = unsafe {
        std::mem::transmute::<_, &'static mut cardworder::cardputer_hal::screen::display::CardputerDisplay<'static>>(
            Box::leak(Box::new(parts.display)),
        )
    };

    // HAL stays on Core 0 (SPI is core-affine)
    let mut hal = parts.hal;

    // Build UI with direct display ownership
    log::info!("boot step 4: CardworderUi::build");
    let mut ui = CardworderUi::build(parts.framebuffer, display);

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
        name: Some(unsafe { CStr::from_bytes_with_nul_unchecked(b"runtime\0") }),
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
        );
        runtime.run();
    });

    // ---- Core 0: keyboard poll + draw loop + Core0Action execution ----
    let free_heap = unsafe { esp_idf_sys::heap_caps_get_free_size(esp_idf_sys::MALLOC_CAP_DEFAULT) };
    let total_heap = unsafe { esp_idf_sys::heap_caps_get_total_size(esp_idf_sys::MALLOC_CAP_DEFAULT) };
    log::info!("boot: free heap {}KB / {}KB, entering Core 0 draw loop", free_heap / 1024, total_heap / 1024);

    let mut input_state = InputState {
        ctrl_pressed: false,
        shift_pressed: false,
        opt_pressed: false,
        alt_pressed: false,
        fn_pressed: false,
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

        // 3. Execute Core0Actions from snapshot (runs HAL on Core 0)
        if got_new_snapshot {
            if let Some(ref snapshot) = current_snapshot {
                if let Some(action) = snapshot.action() {
                    let result = execute_core0_action(&mut hal, action, &mut ntp_instance);
                    // Track WiFi connected state for top bar icon
                    match &result {
                        Core0Result::WifiConnected { .. } => wifi_connected = true,
                        Core0Result::WifiStopped => wifi_connected = false,
                        _ => {}
                    }
                    let _ = core0_msg_tx.send(Msg::Core0Result(result));
                }
            }
        }

        // 4. Draw only when needed
        let now_us = unsafe { esp_idf_svc::sys::esp_timer_get_time() as u64 };
        let timer_tick = (now_us - last_draw_us) >= 1_000_000;
        let should_draw = got_new_snapshot || timer_tick;

        if should_draw {
            if let Some(ref snapshot) = current_snapshot {
                if got_new_snapshot {
                    // Full redraw: clear without dirtying, draw content, mark previous extent
                    ui.clear_no_dirty(Rgb565::BLACK);
                    snapshot.draw(&mut ui);
                    if snapshot.needs_top_line() {
                        ui.draw_top_line(&input_state, &last_pressed, wifi_connected);
                    }
                    // Ensure previously-drawn rows get flushed (to clear old content on display)
                    ui.mark_rows_dirty(0, prev_dirty_max_y);
                } else {
                    // Clock-only redraw: only update top bar (12 rows)
                    if snapshot.needs_top_line() {
                        ui.draw_top_line(&input_state, &last_pressed, wifi_connected);
                    }
                }
                ui.flip_buffer();
                // Remember this frame's dirty extent for next frame
                prev_dirty_max_y = 134; // will be refined once we see actual bbox
            }
            last_draw_us = now_us;
        }

        // 5. Yield
        FreeRtos::delay_ms(8);
    }
}
