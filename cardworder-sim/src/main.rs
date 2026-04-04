mod executor;
mod keyboard;
mod sdl_display;
mod storage;

use cardworder_core::runtime::Runtime;
use cardworder_core::screen::Snapshot;
use cardworder_core::screens::main_menu::MainMenuScreen;
use cardworder_core::types::Msg;
use cardworder_core::ui::cardworder_ui::CardworderUi;
use cardworder_core::ui::framebuffer::{
    CardworderFramebuffer, DISPLAY_HEIGHT, DISPLAY_WIDTH,
};
use cardworder_core::types::dispatch_action as dispatch;
use embedded_graphics::pixelcolor::Rgb565;
use embedded_graphics::prelude::RgbColor;
use embedded_graphics_framebuf::FrameBuf;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl_display::{SdlDisplay, SCALE};

use executor::SimExecutor;
use keyboard::SimKeyboard;

type SimUi = CardworderUi<CardworderFramebuffer>;

fn local_time_hms() -> (i32, i32, i32) {
    use chrono::Timelike;
    let now = chrono::Local::now();
    (now.hour() as i32, now.minute() as i32, now.second() as i32)
}

fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    keyboard::log_keymap();

    // ── SDL setup ─────────────────────────────────────────────────────────────
    let sdl_context = sdl2::init().expect("SDL2 init failed");
    let video = sdl_context.video().expect("SDL2 video init failed");

    let window = video
        .window(
            "CardWorder Simulator",
            DISPLAY_WIDTH as u32 * SCALE,
            DISPLAY_HEIGHT as u32 * SCALE,
        )
        .position_centered()
        .build()
        .expect("Failed to create window");

    let canvas = window
        .into_canvas()
        .accelerated()
        .build()
        .expect("Failed to create canvas");

    let texture_creator = Box::leak(Box::new(canvas.texture_creator()));
    let sdl_display = SdlDisplay::new(canvas, texture_creator);

    // ── Framebuffer + UI ──────────────────────────────────────────────────────
    let fb_backend = CardworderFramebuffer::new(Rgb565::BLACK);
    let framebuf = FrameBuf::new(fb_backend, DISPLAY_WIDTH, DISPLAY_HEIGHT);
    let display_box = Box::new(sdl_display);
    let mut ui: SimUi = CardworderUi::build(framebuf, display_box, local_time_hms);

    // ── Channels ──────────────────────────────────────────────────────────────
    let (msg_tx, msg_rx) = std::sync::mpsc::channel::<Msg>();
    let (state_tx, state_rx) = std::sync::mpsc::channel::<Snapshot>();

    let kb_msg_tx = msg_tx.clone();
    let core0_msg_tx = msg_tx.clone();

    // ── Executor (Core0 equivalent) ───────────────────────────────────────────
    let mut executor = SimExecutor::new(msg_tx.clone());

    // ── Runtime thread ────────────────────────────────────────────────────────
    let runtime_msg_tx = msg_tx;
    let runtime_state_tx = state_tx;

    std::thread::spawn(move || {
        let runtime = Runtime::new(
            Box::new(MainMenuScreen::default()),
            msg_rx,
            runtime_msg_tx,
            runtime_state_tx,
            |ms| std::thread::sleep(std::time::Duration::from_millis(ms as u64)),
            |task| {
                std::thread::spawn(task);
            },
        );
        runtime.run();
    });

    // ── Event loop ────────────────────────────────────────────────────────────
    let mut event_pump = sdl_context.event_pump().expect("Failed to get event pump");
    let mut sim_keyboard = SimKeyboard::new();
    let mut current_snapshot: Option<Snapshot> = None;
    let mut last_draw = std::time::Instant::now();
    let mut input_state = cardworder_core::input::keyboard::InputState {
        ctrl_pressed: false,
        shift_pressed: false,
        opt_pressed: false,
        alt_pressed: false,
        fn_pressed: false,
        fn_locked: true,
        shift_locked: false,
        fn_last_release_us: 0,
        shift_last_release_us: 0,
        lang: cardworder_core::input::keyboard::InputLanguage::En,
    };
    let mut last_pressed = None;

    'running: loop {
        // 1. Handle SDL events
        for event in event_pump.poll_iter() {
            match &event {
                Event::Quit { .. }
                | Event::KeyDown { keycode: Some(Keycode::F10), .. } => break 'running,
                _ => {
                    if let Some(key_msg) = sim_keyboard.handle_event(&event) {
                        input_state = key_msg.input_state;
                        last_pressed = key_msg.pressed;
                        let _ = kb_msg_tx.send(Msg::Key(key_msg));
                    }
                }
            }
        }

        // 2. Drain snapshot channel
        let mut got_new_snapshot = false;
        while let Ok(snapshot) = state_rx.try_recv() {
            current_snapshot = Some(snapshot);
            got_new_snapshot = true;
        }

        // 3. Draw
        let timer_tick = last_draw.elapsed() >= std::time::Duration::from_secs(1);
        let should_draw = got_new_snapshot || timer_tick;

        if should_draw {
            if let Some(ref snapshot) = current_snapshot {
                if got_new_snapshot {
                    ui.clear_no_dirty(Rgb565::BLACK);
                    snapshot.draw(&mut ui);
                    if snapshot.needs_top_line() {
                        ui.draw_top_line(&input_state, &last_pressed, false, 100);
                    }
                    ui.mark_rows_dirty(0, DISPLAY_HEIGHT - 1);
                } else {
                    // clock-only tick
                    if snapshot.needs_top_line() {
                        ui.draw_top_line(&input_state, &last_pressed, false, 100);
                    }
                }
                ui.flip_buffer();
            }
            last_draw = std::time::Instant::now();
        }

        // 4. Execute Core0Actions
        if got_new_snapshot {
            if let Some(ref mut snapshot) = current_snapshot {
                if let Some(action) = snapshot.action() {
                    let result = dispatch(&mut executor, action);
                    let _ = core0_msg_tx.send(Msg::Core0Result(result));
                }
            }
        }

        std::thread::sleep(std::time::Duration::from_millis(8));
    }
}
