# ESP32-S3 UI Architecture — Refactoring Reference

## Context

- **Device**: ESP32-S3, dual Xtensa LX7 cores
- **Language**: Rust, `std` available via esp-idf
- **Key deps**: `esp-idf-hal 0.46`, `esp-idf-svc 0.52`, `mipidsi 0.8`, `embedded-graphics 0.8`,
  `heapless 0.9`, `embedded-sdmmc 0.9`, `serde/serde_json`
- **Pattern**: Elm architecture (Model → Update → View) adapted for dual-core embedded

---

## Core Assignment

| Core | Role | Tasks |
|------|------|-------|
| Core 0 (PRO_CPU) | UI | keyboard polling, draw loop |
| Core 1 (APP_CPU) | Logic | message processing, state mutation, background tasks |

Core 0 **never blocks** and **never mutates** application state.
Core 1 **never touches** the display directly.

---

## Threading Model

```
Core 0 (main thread, prio 6)          Core 1 (spawned thread, prio 5)
────────────────────────────           ────────────────────────────────
loop {                                 loop {
  keyboard.poll()                        msg = msg_rx.recv()   // blocks, 0 CPU
    → msg_tx.try_send(Msg::Key)          state = screen.handle_msg(msg)
                                         execute_command(cmd)
  while state_rx.try_recv()             state_tx.try_send(snapshot)
    → local_snapshot = s             }
                                     
  drawer.tick()        // blink       Background task threads (Core 1, prio 4)
  screen.draw(snapshot)              ────────────────────────────────────────
}                                    spawned by Command::SpawnTask
                                     report back via msg_tx clone
```

**Channel types**:
- `msg_tx / msg_rx`: `mpsc::sync_channel::<Msg>(16)` — keyboard + task results → Core 1
- `state_tx / state_rx`: `mpsc::sync_channel::<Snapshot>(2)` — state snapshot → Core 0

---

## Message Type (`types.rs`)

```rust
pub enum Msg {
    Key(KeyEvent),
    TaskProgress { id: u8, percent: u8 },
    TaskDone     { id: u8 },
    TaskError    { id: u8, msg: &'static str },
    Tick,                        // optional periodic heartbeat
}
```

---

## Command Type (`types.rs`)

Returned by `handle_msg()`. The runtime on Core 1 executes these — components never act directly.

```rust
pub enum Command {
    None,
    SpawnTask {
        id: u8,
        task: Box<dyn FnOnce(TaskHandle) + Send + 'static>,
    },
    SwitchTo(Box<dyn Screen + Send>),
    UpdateShared(SharedStateUpdate),
    FieldSubmitted { label: &'static str, value: heapless::String<64> },
    Multiple(Vec<Command>),
}
```

---

## TaskHandle (`types.rs`)

Passed into background tasks so they can report progress without knowing about channels.

```rust
#[derive(Clone)]
pub struct TaskHandle {
    pub id: u8,
    pub tx: mpsc::SyncSender<Msg>,
}

impl TaskHandle {
    pub fn progress(&self, pct: u8) {
        self.tx.try_send(Msg::TaskProgress { id: self.id, percent: pct }).ok();
    }
    pub fn done(self) {
        self.tx.try_send(Msg::TaskDone { id: self.id }).ok();
    }
    pub fn error(self, msg: &'static str) {
        self.tx.try_send(Msg::TaskError { id: self.id, msg }).ok();
    }
}
```

---

## Shared State (`types.rs`)

State shared across all screens. Only Core 1 mutates it. Passed as `&SharedState` to
`handle_msg` and `on_mount`.

```rust
#[derive(Clone, Default)]
pub struct SharedState {
    pub volume:     u8,
    pub sd_mounted: bool,
    // add fields as needed
}

pub enum SharedStateUpdate {
    SetVolume(u8),
    SetSdMounted(bool),
}
```

---

## Screen Trait (`screen.rs`)

Top-level views. One screen is active at a time. `handle_msg` runs on Core 1.
`draw` runs on Core 0 from a snapshot — it never touches the live screen object.

```rust
pub trait Screen {
    /// Called once when this screen becomes active (Core 1)
    fn on_mount(&mut self, shared: &SharedState) -> Command {
        Command::None
    }

    /// Core 1: process message, mutate self, return command
    fn handle_msg(&mut self, msg: Msg, shared: &SharedState) -> Command;

    /// Core 0: draw from snapshot. Must never block. Never called with live state.
    /// Implement on the Snapshot type, not on Screen directly (see Snapshot pattern below).
    fn draw(&self, display: &mut Display, shared: &SharedState);
}
```

---

## Snapshot Pattern

After every `handle_msg`, Core 1 serializes drawable state into a plain `Clone` struct
and sends it to Core 0. Core 0 draws only from this struct — it never accesses the
live screen object.

```rust
// Every screen defines its own snapshot
#[derive(Clone)]
pub struct MyScreenSnapshot {
    pub cursor:   usize,
    pub items:    heapless::Vec<heapless::String<32>, 16>,
    pub loading:  bool,
    pub load_pct: u8,
}

// Core 1 sends this after handle_msg:
state_tx.try_send(Box::new(snapshot)).ok();

// Core 0 calls a draw function on the snapshot type:
impl MyScreenSnapshot {
    pub fn draw(&self, display: &mut Display) { ... }
}
```

---

## Widget Trait (`widget.rs`)

Sub-components embedded inside screens (input fields, progress bars, menus, etc.).
A widget is not a screen — it is owned and driven by its parent screen.

```rust
pub trait Widget {
    type Snapshot: Clone + Send;

    /// Core 1: handle message, mutate self, return command
    fn handle_msg(&mut self, msg: &Msg) -> Command;

    /// Core 1: produce snapshot for Core 0
    fn snapshot(&self) -> Self::Snapshot;
}
```

The parent screen's `handle_msg` delegates to the widget and forwards the returned `Command`:

```rust
fn handle_msg(&mut self, msg: Msg, shared: &SharedState) -> Command {
    let cmd = self.my_widget.handle_msg(&msg);
    // inspect or forward cmd
    cmd
}
```

---

## InputField Widget (`widgets/input_field.rs`)

### Core 1 — owned state

```rust
pub struct InputField<const N: usize> {
    pub buf:        heapless::Vec<u8, N>,  // store as bytes; ASCII = byte == char
    pub cursor:     usize,                  // byte index
    pub focused:    bool,
    pub validation: ValidationState,
    label:          &'static str,
    validator:      fn(&str) -> ValidationState,
}

#[derive(Clone, PartialEq, Debug)]
pub enum ValidationState {
    Empty,
    Valid,
    Invalid(&'static str),
}
```

Key rules in `handle_msg`:
- Only processes keys when `self.focused == true`
- Char → push byte, advance cursor, re-validate
- Backspace → rebuild buffer without byte at `cursor - 1`
- Enter + Valid → return `Command::FieldSubmitted`
- Left/Right → move cursor, clamp to `[0, buf.len()]`

### Core 0 — draw-only state (NOT in snapshot)

```rust
pub struct InputFieldDrawer {
    blink_visible: bool,
    last_blink:    std::time::Instant,
    blink_period:  std::time::Duration,  // typically 530ms
}

impl InputFieldDrawer {
    /// Call once per frame before drawing
    pub fn tick(&mut self) {
        if self.last_blink.elapsed() >= self.blink_period {
            self.blink_visible = !self.blink_visible;
            self.last_blink = std::time::Instant::now();
        }
    }
}
```

**Cursor blink is never driven by a message.** It is computed from `Instant::now()` on
Core 0 only. Core 1 has no involvement.

### Snapshot

```rust
#[derive(Clone)]
pub struct InputFieldSnapshot<const N: usize> {
    pub buf:        heapless::Vec<u8, N>,
    pub cursor:     usize,
    pub focused:    bool,
    pub validation: ValidationState,
    pub label:      &'static str,
}
```

### Draw logic (Core 0)

```
border color  = gray (Empty) | green (Valid) | red (Invalid)
border width  = 1px (unfocused) | 2px (focused)
cursor        = drawn only if focused AND blink_visible
error text    = drawn below field if ValidationState::Invalid
```

---

## Runtime (`runtime.rs`)

Lives entirely on Core 1. Drives the active screen and executes commands.

```rust
fn run(initial_screen: Box<dyn Screen + Send>, msg_rx, state_tx, shared) {
    let cmd = screen.on_mount(&shared);
    execute(cmd, ...);

    for msg in msg_rx.iter() {          // blocks; 0 CPU when idle
        let cmd = screen.handle_msg(msg, &shared);
        execute(cmd, ...);
        state_tx.try_send(snapshot).ok();
    }
}

fn execute(cmd, msg_tx, screen, shared) {
    match cmd {
        Command::None => {}

        Command::SpawnTask { id, task } => {
            // pin to Core 1, prio 4
            ThreadSpawnConfiguration { pin_to_core: Some(Core::Core1), priority: 4, .. }
                .set().ok();
            let handle = TaskHandle { id, tx: msg_tx.clone() };
            std::thread::spawn(move || task(handle));
        }

        Command::SwitchTo(mut new_screen) => {
            let mount_cmd = new_screen.on_mount(shared);
            *screen = new_screen;
            execute(mount_cmd, ...);  // handle mount command too
        }

        Command::UpdateShared(update) => { /* apply to shared */ }

        Command::FieldSubmitted { .. } => { /* parent already handled this */ }

        Command::Multiple(cmds) => {
            for c in cmds { execute(c, ...); }
        }
    }
}
```

---

## Thread Pinning Boilerplate

```rust
fn spawn_on_core(core: Core, name: &'static [u8], stack: usize, prio: u8, f: impl FnOnce() + Send + 'static) {
    ThreadSpawnConfiguration {
        name: Some(name),
        stack_size: stack,
        priority: prio,
        pin_to_core: Some(core),
        ..Default::default()
    }.set().unwrap();
    std::thread::spawn(f);
}

// Core 1 runtime
spawn_on_core(Core::Core1, b"runtime\0", 16384, 5, move || Runtime::run(...));

// Background tasks (spawned from within runtime, also Core 1)
spawn_on_core(Core::Core1, b"task\0", 8192, 4, move || task(handle));
```

---

## SPI Bus Sharing

Display and SD card share the SPI bus. Both cores could trigger a transfer (Core 0 for
display, Core 1 for SD). Wrap the bus driver in `Arc<Mutex<SpiDriver>>`.

```rust
let spi = Arc::new(Mutex::new(spi_driver));
let spi_display = Arc::clone(&spi);
let spi_sd      = Arc::clone(&spi);
```

**Rule**: lock → transfer → release immediately. Never hold the lock across a sleep or
a channel operation.

---

## Priority Table

| Thread | Core | Priority | Stack |
|--------|------|----------|-------|
| Keyboard + draw loop | 0 | 6 | 8 kB |
| Runtime (logic) | 1 | 5 | 16 kB |
| Background tasks | 1 | 4 | 8 kB |
| SD logic (if separate thread) | 1 | 4 | 8 kB |

Keep all application threads below 23 (WiFi/BT stack priority).

---

## File Layout (suggested)

```
src/
  main.rs              — wiring: channels, thread spawn, draw loop
  types.rs             — Msg, Command, TaskHandle, SharedState
  runtime.rs           — Runtime::run(), execute()
  screen.rs            — Screen trait, Display type alias
  widget.rs            — Widget trait
  screens/
    mod.rs
    file_browser.rs    — FileBrowser, FileBrowserSnapshot
    login.rs           — LoginScreen, LoginSnapshot
    main_menu.rs
  widgets/
    mod.rs
    input_field.rs     — InputField<N>, InputFieldSnapshot<N>, InputFieldDrawer
    progress_bar.rs
    list.rs
  hal/
    keyboard.rs        — your existing HAL
    display.rs         — your existing HAL
    sdcard.rs          — your existing HAL
```

---

## Refactoring Checklist

- [ ] Define `Msg`, `Command`, `TaskHandle`, `SharedState`, `SharedStateUpdate` in `types.rs`
- [ ] Define `Screen` trait and `Display` type alias in `screen.rs`
- [ ] Define `Widget` trait in `widget.rs`
- [ ] Implement `Runtime::run()` and `execute()` in `runtime.rs`
- [ ] Move each existing screen into `screens/` implementing `Screen`
- [ ] Extract `InputField` widget into `widgets/input_field.rs`
- [ ] Add `InputFieldDrawer` to draw loop (Core 0 side)
- [ ] Wrap SPI bus in `Arc<Mutex<_>>` if display and SD share a bus
- [ ] Replace any direct state mutation in draw path with snapshot reads
- [ ] Ensure no `thread::sleep` or blocking calls in the Core 0 draw loop
