# TASK ARCHIVE: Menu drawn under top bar — fix layout

## METADATA

| Field | Value |
|-------|--------|
| **Task ID** | DEV-0002 |
| **Status** | completed |
| **Complexity** | Level 1 |
| **Started** | 2026-03-17 |
| **Type** | bugfix / layout |
| **Priority** | high |
| **Repository** | card-worder |

## SUMMARY

The main menu was drawing on top of the top bar (first line hidden or overlapping). Two fixes were applied: (1) draw order and content offset so the menu viewport starts below the top bar; (2) use the vertical center of each line rect when drawing Center-aligned glyphs so icons/text do not extend into the top bar area.

## REQUIREMENTS

- Top bar visible and separate from menu content.
- Menu content fully visible below the top bar (no first line under/on the bar).

## IMPLEMENTATION

1. **`src/ui/cardworder_ui.rs`**  
   - Added `TOP_BAR_HEIGHT: u32 = 12` for the status line + separator height.

2. **`src/logic/view_manager.rs`**  
   - Draw top bar first when `is_need_top_line()`, then `current_view.draw()`, so the view draws in the area below the bar.

3. **`src/logic/views/render.rs`**  
   - `compose_form` now takes `content_top_y: i32`; visible line rects use `(y - scroll_offset) + content_top_y` so content is placed below the top bar.  
   - `draw_elements_line`: compute `y_center = line_top_y + line_height / 2` and use it for Center-aligned icon and text; use `line_top_y` / `line_top_y + line_height` for Top/Bottom so glyphs stay within the line slot and do not draw into the top bar.

4. **`src/logic/views/main_menu.rs`**  
   - Viewport height = screen height − `TOP_BAR_HEIGHT`; calls `compose_form(..., viewport_height, TOP_BAR_HEIGHT as i32, ui)` so menu content starts at y = 12.

## TESTING

Manual verification on device: top bar and menu list both visible; first menu line below the bar with no overlap.

## LESSONS LEARNED

- With `VerticalPosition::Center`, the draw point is the glyph’s vertical center; passing the line’s top y caused tall glyphs (e.g. IconsHuge) to extend into the top bar. Using the line rect’s vertical center for Center-aligned elements keeps all drawing below the top bar.

## REFERENCES

- Memory Bank: `memory-bank/systemPatterns.md` (TOP_BAR_HEIGHT usage)
- Related: DEV-0001 (platform memory pattern)
