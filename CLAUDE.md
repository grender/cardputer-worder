# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

CardWorder is a spaced-repetition vocabulary learning app using the FSRS algorithm. It targets two platforms from a shared Rust workspace:

- **`cardworder/`** — ESP32 embedded app for the M5Stack Cardputer (Xtensa, `esp` toolchain, ESP-IDF v5.1.4). Drives an ST7789V2 display, matrix keyboard, SD card, and Wi-Fi.
- **`console-learner/`** — Desktop/terminal app for reviewing words from the console.
- **`fsrs-core/`** — Platform-agnostic library (`no_std`-compatible via `default-features = false`) shared by both apps. Contains word card models, FSRS scheduling logic, storage traits, and vocabulary management.

## Build Commands

### console-learner (standard Rust)
```bash
cargo build -p console-learner
cargo run -p console-learner
```

### fsrs-core (library, tests)
```bash
cargo build -p fsrs-core
cargo test -p fsrs-core
```

### cardworder (ESP32 — requires `espup` / Xtensa toolchain)
Must be built from inside `cardworder/` because it uses its own `rust-toolchain.toml` (`channel = "esp"`):
```bash
cd cardworder
cargo build              # build main binary
cargo build --bin hal_test_screen   # build a HAL test binary
```
Flash to device:
```bash
cd cardworder
espflash flash target/xtensa-esp32s3-espidf/debug/cardworder --monitor
```

## Architecture Notes

### cardworder (ESP app)
- `cardputer_hal/` — hardware abstraction: display (ST7789V2 via SPI + framebuffer), keyboard (matrix scan), SD card (`embedded-sdmmc`), Wi-Fi
- `logic/` — view system: `View` trait, `ViewManager` for screen navigation, individual views (start, main_menu, render)
- `ui/` — top-level UI orchestration (`cardworder_ui`)
- `bin/` — main entry point and individual HAL test binaries (`hal_test_screen`, `hal_test_keyboard`, `hal_test_wifi`, `hal_test_sd`)

### fsrs-core
- `models.rs` — `WordCard`, `CardId` types
- `fsrs.rs` — FSRS scheduling helpers, `Rating`, `is_card_due`
- `vocabulary.rs` — `VocabularyManager`, `Stats`
- `storage.rs` — `Storage` trait for persistence
- `error.rs` — `VocabularyError`

### console-learner
- `app.rs` — application logic
- `storage.rs` — JSON file-based storage implementation
- `ui/` — terminal menu and review UI
- `words.json` — word data; `import_obsidian_words.py` imports words from Obsidian notes

## Key Constraints

- The `cardworder` crate pins older versions of `esp-idf-svc` (0.52.1) because Embassy doesn't support the latest Xtensa Rust toolchain.
- `mipidsi` is pinned at 0.8.0 due to breaking API changes in newer versions.
- ESP build uses 8MB flash, 240 MHz CPU, and a custom partition table (`partition-table.csv`).
- `fsrs-core` has a `std` feature (on by default); disable it for `no_std` embedded use.
