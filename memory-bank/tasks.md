# Tasks: Scrollable Minimalist UI for Cardputer

## Overview
Implement a minimalist, scrollable form rendering system for the Cardputer UI, using only primitive elements, explicit vertical stacking, and a dynamic compose step for small screens.

---

## Implementation Plan & Checklist

### File Structure & Responsibilities

#### 1. `src/logic/views/fonts.rs`
- [x] Move all font-related logic here:
  - `CardFont` enum and font height constants
  - Functions for measuring line height for `UiLineType`/`UiLineElement`
  - Multiline text measurement and wrapping logic

#### 2. `src/logic/views/render.rs`
- [x] Move all render, compose, and scroll logic here:
  - `compose_form` (calculates line heights, assigns rectangles, returns visible lines)
  - Scroll state and offset management
  - Scroll up/down logic (adjusts offset by line height, clamps to valid range)
  - Filtering visible lines for rendering
  - Calls to low-level drawing methods in `cardworder_ui.rs`
- [x] **NEW: Scroll Bar Implementation**
  - `ScrollBar` struct with smart sizing and positioning
  - Visual feedback showing content vs. viewport ratio
  - Auto-hide when content fits in viewport
  - Right-aligned 4-pixel wide scroll bar

#### 3. `src/logic/views/mod.rs`
- [x] Re-export and coordinate between `fonts.rs`, `render.rs`, and view modules
- [x] Define/organize `UiLineType`, `UiLineElement`, etc.

#### 4. `src/logic/views/main_menu.rs`, `src/logic/views/start.rs`
- [x] Use the new scrollable form logic for actual views
- [x] Add test forms and validate integration
- [x] **Enhanced with 12 menu items** to demonstrate scrolling

---

## Implementation Checklist

- [x] **Create `fonts.rs`** and move all font, line height, and multiline logic there.
- [x] **Create `render.rs`** and move all compose, scroll, and render logic there.
- [x] **Update `mod.rs`** to use the new modules and organize UI element types.
- [x] **Update views** (`main_menu.rs`, `start.rs`) to use the new logic.
- [x] **Test** with forms of varying content and line heights.
- [x] **Document** the new structure and usage in code and Memory Bank.
- [x] **Implement Scroll Bar** with visual feedback for content scrolling.

---

## Scroll Bar Features Implemented

### Visual Indicators
- **Scroll Bar Track**: Dark gray background spanning viewport height
- **Scroll Bar Thumb**: Light gray slider showing visible content ratio
- **Smart Sizing**: Thumb size proportional to visible content
- **Position Feedback**: Thumb position indicates scroll location

### Smart Behavior
- **Auto-hide**: Only appears when content > viewport
- **Proportional**: Thumb size reflects content visibility
- **Minimum Size**: Never smaller than 8 pixels for usability
- **Right-aligned**: 4-pixel width on right edge

### User Experience
- **Spatial Awareness**: Users know their position in content
- **Content Discovery**: Users see how much more content is available
- **Navigation Efficiency**: Users can estimate scroll distance
- **Visual Feedback**: Clear indication of scroll state

---

**Principle:**  
- All UI logic for forms, scrolling, composition, and measurement lives in `src/logic/views/` (in `fonts.rs` and `render.rs`).
- `cardworder_ui.rs` is only for low-level drawing primitives.
- **Scroll bar provides intuitive visual feedback for content navigation.**

---

## Files Created/Modified

- `src/logic/views/render.rs` - Added ScrollBar struct and rendering
- `src/ui/cardworder_ui.rs` - Added fill_rect method for scroll bar drawing
- `src/logic/views/main_menu.rs` - Enhanced with 12 menu items for scrolling demo
- `SCROLL_BAR_README.md` - Comprehensive documentation of scroll bar functionality

---

## New Feature: InputText UiLineElement

### Overview
Added a new `InputText` variant to the `UiLineElement` enum to support text input fields with cursor positioning.

### Implementation Details

#### 1. New UiLineElement Variant
- **Type**: `InputText(&'static str, CardFont, ThemeColor, u32)`
- **Parameters**: 
  - `text`: The input text content
  - `font`: Font to use for rendering
  - `color`: Text color
  - `cursor_position`: Character position where cursor should appear

#### 2. Updated Measurement Logic
- **File**: `src/logic/views/fonts.rs`
- **Function**: `measure_element_height()` now handles `InputText` elements
- **Behavior**: InputText elements contribute the same height as regular text elements

#### 3. Enhanced Rendering
- **File**: `src/logic/views/render.rs`
- **Function**: `draw_elements_line()` now renders InputText elements
- **Features**:
  - Renders the input text using existing text drawing
  - Calculates cursor position based on text width
  - Draws a visual cursor indicator

#### 4. New UI Methods
- **File**: `src/ui/cardworder_ui.rs`
- **Methods Added**:
  - `measure_text_width()`: Calculates text width up to cursor position
  - `draw_cursor()`: Renders a 2-pixel wide vertical cursor line

#### 5. Demo Integration
- **File**: `src/logic/views/main_menu.rs`
- **Added**: Test input field showing "Hello World" with cursor at position 5
- **Purpose**: Demonstrates the new InputText element functionality

### Usage Example
```rust
UiLineElement::InputText("Hello World", CardFont::Medium, ThemeColor::Selected, 5)
```

This creates an input field displaying "Hello World" with a cursor positioned after the 5th character.

### Technical Features
- **Cursor Visualization**: 2-pixel wide vertical line indicator
- **Position Calculation**: Smart cursor positioning based on font metrics
- **Font Support**: Works with all existing CardFont variants
- **Color Integration**: Uses existing ThemeColor system
- **Scrollable**: Integrates with existing scroll and compose logic

### Future Enhancements
- **Dynamic Text**: Support for mutable text content
- **Input Handling**: Keyboard input processing for text editing
- **Cursor Blinking**: Animated cursor for better visibility
- **Text Selection**: Highlighting selected text ranges
