# Tech Context

- **Platform:** ESP32 (Cardputer)
- **Programming Language:** Rust (2021 edition)
- **Key Dependencies:**
  - `esp-idf-svc`, `esp-idf-hal`, `esp-idf-sys` for ESP32 hardware support
  - `embedded-hal`, `embedded-graphics` for hardware abstraction and display
  - `serde`, `serde_json` for data serialization
  - `anyhow`, `log` for error handling and logging
  - `rs-fsrs` for implementing the FSRS spaced repetition algorithm
- **Build System:** Cargo, with custom profiles for release and development
- **Time Sync:** NTP via WiFi
- **UI:** Custom UI module for device screen and keyboard

## UI Architecture

- **Manual Construction:** All forms and screens are built manually from primitive elements.
- **Vertical Stacking:** The primary layout is a vertical stack of lines; horizontal grouping is only via explicit element vectors.
- **Scrolling Support:** Forms support scrolling via a scroll offset and dynamic compose step.
- **Dynamic Layout:** Line heights are determined at compose time, allowing for variable-height content.
- **Viewport Rendering:** Only lines intersecting the viewport are rendered, optimizing for the small screen and limited resources.

## Multiline Text Rendering

- **Dynamic Measurement:** The compose step includes a text measurement function that calculates the number of lines and total height for each multiline text element, based on font and available width.
- **Rendering:** Multiline text is rendered line by line, with correct vertical offsets, supporting both explicit line breaks and automatic word wrapping.

## Device Display

- **Screen Size:** 240 x 135 pixels (landscape orientation)
- **Implications:** All UI layouts, font sizes, and scrolling logic are designed to fit and perform optimally within this resolution.

## Platform Constraints & Mandatory Rules

- **Target Platform:** ESP32 (Cardputer) is a resource-constrained, embedded device.
- **Limited Standard Library:** Many standard Rust methods and crates are unavailable or only partially supported.
- **Mandatory Double-Check:** Always verify if a method, crate, or feature is supported on ESP32 before use.
- **Defensive Coding:** Prefer explicit error handling and fallback logic for any platform-dependent functionality.
- **Performance Awareness:** Optimize for low memory and CPU usage; avoid heavy abstractions and unnecessary allocations.

