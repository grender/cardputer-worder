# System Patterns

- **Modular Architecture:** The codebase is organized into modules for hardware abstraction (`cardputer_hal`), user interface (`ui`), and application logic (`logic`).
- **Spaced Repetition Algorithm:** Core logic implements repetition scheduling for vocabulary learning using the FSRS algorithm via the `rs-fsrs` crate.
- **NTP Integration:** System synchronizes time over WiFi to support accurate scheduling.
- **Hardware Abstraction:** Separate modules for screen, keyboard, WiFi, and SD card access.
- **Error Handling:** Custom trait for logging and handling errors (`ResultExt`).
- **Defensive Programming:** Code should always check for platform compatibility and handle missing features gracefully, as the ESP32 platform is limited and many standard methods may not be available.

## UI Design Approach

The UI is built around a minimalist, explicit pattern suitable for embedded devices with small screens and limited resources. Forms are constructed manually from primitive elements, with vertical stacking as the primary layout. Scrolling is supported for forms that exceed the screen height, using a compose step to dynamically calculate line heights and a viewport to render only visible content.

## UI/UX Constraints

- The device screen is small, so forms are scrollable.
- All UI is constructed from primitive elements for maximum efficiency and predictability.
- No high-level UI abstractions, builders, or event/callback systems are used.
- The user navigates forms using simple up/down controls, with smooth scrolling and clear focus.

- **Scrollable Form Pattern:** Forms are vertical stacks of lines, each line composed of primitive elements. When the form exceeds the viewport, a scroll offset is maintained and only visible lines are rendered.
- **Compose Step:** Before rendering, the form is "composed"—each line's height is dynamically measured, and its position is calculated based on the current scroll offset and viewport size. This enables efficient rendering and smooth scrolling, even with variable-height lines.
- **No Declarative or High-Level UI:** All UI is constructed manually in code, with no macros, builders, or high-level abstractions. This ensures maximum control and compatibility with the ESP32's constraints.

| Feature                | Supported? | Notes                                 |
|------------------------|------------|---------------------------------------|
| Declarative/Builder    | ❌         | Manual construction only              |
| High-level UI Elements | ❌         | Only primitive elements allowed       |
| Actions/Callbacks      | ❌         | No event handling                     |
| Vertical stacking      | ✅         | Via manual Vec of lines               |
| Horizontal stacking    | ✅         | Only via UiLineType::Elements         |
| Scrolling              | ✅         | Via scroll_offset and compose step    |
| Dynamic line heights   | ✅         | Measured at compose time              |
| Viewport rendering     | ✅         | Only visible lines are drawn          |

- **Multiline Text Support:** Each `UiLineElement::Text` can contain text that wraps or includes explicit line breaks. The number of lines and rendered height are determined dynamically during the compose step, using font metrics and available width. This enables accurate layout, scrolling, and rendering for variable-length text content.

## Monospace Word Wrapping for Multiline Text

- **Rationale:** All UI fonts are monospace, so every character has the same width. This allows for simple, efficient word wrapping and line measurement, ideal for embedded systems.
- **Algorithm:** When rendering or measuring a `UiLineElement::Text`, the string is split into lines that fit within the maximum number of characters per line. The algorithm prefers to break at spaces, but will break mid-word if necessary. This is done using string slices (`&str`) and is UTF-8 safe.
- **Sample Implementation:**

```rust
/// Wraps a string into lines that fit within `max_chars_per_line`.
/// Breaks at spaces when possible, but will break mid-word if necessary.
/// Returns a Vec of &str slices into the original string.
fn wrap_text_monospace<'a>(text: &'a str, max_chars_per_line: usize) -> Vec<&'a str> {
    let mut lines = Vec::new();
    for paragraph in text.split('\n') {
        let mut start_char = 0;
        let chars: Vec<char> = paragraph.chars().collect();
        let total_chars = chars.len();

        while start_char < total_chars {
            let mut end_char = (start_char + max_chars_per_line).min(total_chars);

            // Try to break at the last space within the allowed range
            if end_char < total_chars {
                if let Some(space_pos) = chars[start_char..end_char].iter().rposition(|&c| c == ' ') {
                    if space_pos > 0 {
                        end_char = start_char + space_pos;
                    }
                }
            }

            // Get byte indices for slicing
            let start_byte = paragraph.char_indices().nth(start_char).map(|(i, _)| i).unwrap_or(paragraph.len());
            let end_byte = paragraph.char_indices().nth(end_char).map(|(i, _)| i).unwrap_or(paragraph.len());

            // Skip leading spaces
            let mut line = &paragraph[start_byte..end_byte];
            line = line.trim_start();

            lines.push(line);

            // Move start_char to the next non-space character after end_char
            start_char = end_char;
            while start_char < total_chars && chars[start_char] == ' ' {
                start_char += 1;
            }
        }
    }
    lines
}
```

- **Usage:** Use this function in your compose and render steps to split and measure multiline text elements.
- **Benefits:** No heap allocations for the text itself, only for the vector of slices. UTF-8 safe. Efficient and robust for embedded/ESP32 use.
