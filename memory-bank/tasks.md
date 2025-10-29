# Tasks: Vocabulary Learning System with FSRS Library [BUILD MODE]

## BUILD Mode Status
- [x] VAN Mode complete
- [x] Complexity determined: LEVEL 3 - Intermediate Feature
- [x] Planning complete
- [x] Technology validation complete (VAN QA passed)
- [x] Creative phases complete
- [x] Console UI design clarified
- [x] Phases 1-4 implementation complete
- [x] Workspace structure created
- [x] fsrs-core library compiles
- [x] console-learner application compiles

## Creative Phase Status
- [x] Architecture: Storage Abstraction Design Complete
- [x] Data Model: WordCard Structure Design Complete
- [x] API Design: VocabularyManager Interface Complete

### Creative Phase Documents
- `memory-bank/creative/creative-storage-abstraction.md`
- `memory-bank/creative/creative-data-model.md`
- `memory-bank/creative/creative-api-design.md`
- `memory-bank/creative/creative-console-ui.md` ✅ NEW

## Task Description
Implement a multi-component vocabulary learning system:
1. Console application for learning English words on desktop
2. Common FSRS library abstracting storage and spaced repetition
3. Storage abstraction layer using trait-based design
4. Cross-platform support (console + embedded Cardputer)

## Complexity Assessment
- **Level**: 3 (Intermediate Feature)
- **Type**: Multi-component feature with architectural decisions
- **Components**: Console app, library crate, storage abstraction, integration with embedded app

## Requirements Analysis

### User Requirements
1. **Console application** for learning English words
   - Separate binary from embedded app
   - Provides learning interface for desktop environment
   
2. **Common FSRS library** 
   - Abstract layer for spaced repetition functionality
   - Platform-agnostic design
   - Reusable by both console and embedded applications

3. **Storage abstraction**
   - Abstract from specific storage implementation
   - Trait-based design for flexibility
   - Support for multiple storage backends

4. **Cross-platform library**
   - Works on both console (desktop) and embedded (Cardputer) platforms
   - Conditional compilation for platform-specific features

## Technology Stack

### Core Technologies
- **Language**: Rust 2021 edition
- **Build System**: Cargo workspace
- **FSRS Library**: rs-fsrs v1.2.1 (already in dependencies)
- **Serialization**: serde + serde_json
- **Storage Abstraction**: Trait-based design
- **Platform Detection**: Conditional compilation (#[cfg])

### Dependencies
- rs-fsrs (spaced repetition algorithm)
- serde, serde_json (data serialization)
- chrono (time handling for console app)
- anyhow (error handling)

## Architecture Design

### Project Structure
```
card-worder/
├── Cargo.toml (workspace root)
├── cardworder/ (existing embedded app)
│   └── Cargo.toml
├── fsrs-core/ (new library crate)
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs
│       ├── storage.rs (trait definitions)
│       ├── fsrs.rs (FSRS integration)
│       └── models.rs (data structures)
└── console-learner/ (new console app)
    ├── Cargo.toml
    └── src/
        ├── main.rs
        ├── storage.rs (JSON file implementation)
        └── app.rs (console interface)
```

### Component Breakdown

#### 1. fsrs-core Library
**Purpose**: Platform-agnostic FSRS and storage abstraction

**Modules**:
- `models.rs`: Word card data structures (Card, ReviewLog)
- `storage.rs`: Storage trait definition (Saveable, Retrievable)
- `fsrs.rs`: FSRS algorithm wrapper
- `lib.rs`: Public API and exports

**Features**:
- Trait-based storage abstraction
- FSRS algorithm integration
- Platform-agnostic (works on std and no_std)
- Serialization support

#### 2. console-learner Application
**Purpose**: Desktop console app for learning words

**Features**:
- Text-based quiz interface
- Review scheduling via FSRS
- JSON file storage
- Interactive learning session

**Storage Implementation**: JSON file-based storage

#### 3. cardworder Integration
**Purpose**: Embed FSRS functionality into existing Cardputer app

**Integration Points**:
- Use fsrs-core library
- Implement storage trait for embedded platform
- Add learning/quiz functionality to UI

## Implementation Plan

### Phase 1: Library Setup & Data Models ✅ COMPLETE
- [x] Convert project to Cargo workspace
- [x] Create fsrs-core crate with Cargo.toml
- [x] Define Word and Card models
- [x] Implement serialization with serde
- [x] Create storage trait definitions

**Completed**: Created workspace structure with cardworder, fsrs-core, and console-learner crates. Implemented models, storage traits, and basic FSRS integration.

### Phase 2: FSRS Integration ✅ COMPLETE
- [x] Integrate rs-fsrs library
- [x] Create FSRS wrapper module
- [x] Implement review scheduling logic with full FSRS algorithm
- [x] Add time handling utilities
- [x] Write unit tests for FSRS logic ✅ **39 tests passing**

**Status**: Full FSRS algorithm integration complete - all fields properly updated.

**UPDATE**: Fixed `reps` and `lapses` tracking - now properly increments on each review and tracks lapses for "Again" ratings.

**UPDATE**: Integrated full FSRS algorithm - now properly updates `stability`, `difficulty`, `elapsed_days`, `scheduled_days`, and `state` fields using the rs-fsrs library's `FSRS.next()` method.

### Phase 3: Storage Abstraction ✅ COMPLETE
- [x] Define Storage trait (Saveable, Loadable, etc.)
- [x] Implement in-memory storage for testing
- [x] Add serialization/deserialization helpers
- [ ] Document storage trait interface

### Phase 4: Console Application ✅ COMPLETE
- [x] Create console-learner binary
- [x] Implement JSON file storage (JsonFileStorage struct)
- [x] Create UI module structure (menu, review, list, stats)
- [x] Implement main menu loop with navigation
- [x] Build add word flow (prompt-based)
- [x] Build review session flow (interactive quiz)
- [x] Add list words functionality
- [x] Add statistics display
- [x] Create input/output helpers (read, display, clear screen)

**Status**: Console application compiles and is ready for testing. All core UI flows implemented.

**UI Structure:**
```
console-learner/src/
├── main.rs              // Entry point, menu loop
├── ui/
│   ├── menu.rs          // Menu display and navigation
│   ├── add_word.rs      // Add word flow
│   ├── review_session.rs // Review workflow
│   ├── list_words.rs    // List all words
│   └── stats.rs         // Statistics display
├── storage.rs           // JSON file storage implementation
└── app.rs               // Main application logic
```

**UI Design:** Prompt-based workflow with simple menu
- Main menu: Add word, Review, List, Stats, Quit
- Add word: Simple prompt flow for front/back/examples
- Review: Interactive quiz with FSRS rating input
- List: Show all words with next review dates
- Stats: Display learning statistics

### Phase 5: Embedded Integration
- [ ] Implement storage trait for embedded platform
- [ ] Add card management to Cardputer UI
- [ ] Create learning session views
- [ ] Integrate with existing view manager
- [ ] Test on Cardputer hardware

## Detailed Integration Plan for Embedded App

### Overview
Integrate fsrs-core vocabulary learning library into the cardworder embedded application to enable vocabulary learning on the Cardputer device.

### Current State Analysis

#### ✅ What Exists
1. **fsrs-core library** - Complete with:
   - WordCard models with rs-fsrs integration
   - Storage trait abstraction
   - VocabularyManager API
   - FSRS scheduling logic
   - Error handling

2. **console-learner** - Reference implementation:
   - JsonFileStorage implementation
   - VocabularyManager integration
   - Review session UI
   - Menu-based workflow

3. **cardworder embedded app** - Existing infrastructure:
   - View-based UI system (`CardputerView` trait)
   - Scrollable forms with `UiLineElement`
   - View manager with navigation
   - Input handling (keyboard events)
   - Font rendering system

#### ❌ What's Missing
1. **Embedded Storage Implementation**:
   - No implementation of `Storage` trait for embedded platform
   - Need SD card or flash-based storage
   
2. **Vocabulary Learning Views**:
   - Add word view
   - Review/quiz session view
   - List words view  
   - Statistics view

3. **Integration**:
   - VocabularyManager integration into app state
   - Storage persistence layer
   - View navigation for learning features

### Architecture Design

#### Component Structure
```
cardworder/src/
├── lib.rs
├── bin/
│   └── cardworder.rs (main entry)
├── ui/
│   └── cardworder_ui.rs (low-level drawing)
└── logic/
    ├── mod.rs
    ├── view_manager.rs
    ├── vocabulary_state.rs (NEW - vocabulary app state)
    └── views/
        ├── mod.rs
        ├── main_menu.rs (add vocabulary menu options)
        ├── vocabulary_menu.rs (NEW - vocabulary main menu)
        ├── add_word.rs (NEW - add word flow)
        ├── review_session.rs (NEW - quiz interface)
        ├── list_words.rs (NEW - word list display)
        ├── vocabulary_stats.rs (NEW - statistics)
        ├── fonts.rs
        └── render.rs
```

#### Key Integration Points

1. **VocabularyState Structure**:
   - Wraps VocabularyManager
   - Owns embedded storage implementation
   - Manages app state for learning session

2. **Storage Implementation**:
   - Create `EmbeddedStorage` struct implementing `Storage` trait
   - Use SD card or flash storage
   - Handle serialization/deserialization

3. **View Integration**:
   - Add vocabulary menu to main menu
   - Create dedicated views for each learning operation
   - Integrate with existing view manager

### Detailed Implementation Plan

#### Step 1: Create Embedded Storage Implementation
**File**: `cardworder/src/logic/storage.rs`

**Purpose**: Implement `fsrs_core::Storage` trait for embedded platform

**Design Decisions**:
- Use SD card for primary storage (if available)
- Fall back to RAM-based storage for testing
- JSON format for human-readable data

**Interface**:
```rust
pub struct EmbeddedStorage {
    cards: heapless::Vec<WordCard, MAX_CARDS>,
    storage_path: &'static str,
}

impl Storage for EmbeddedStorage {
    fn save_card(&mut self, card: WordCard) -> Result<(), VocabularyError>;
    fn load_card(&self, id: CardId) -> Result<WordCard, VocabularyError>;
    fn load_all_cards(&self) -> Result<Vec<WordCard>, VocabularyError>;
    fn delete_card(&mut self, id: CardId) -> Result<(), VocabularyError>;
}
```

**Implementation Notes**:
- Use `heapless::Vec` for no_std compatibility
- Implement SD card file operations
- Handle serialization with serde_json
- Add auto-save functionality

#### Step 2: Create Vocabulary State Manager
**File**: `cardworder/src/logic/vocabulary_state.rs`

**Purpose**: Manage vocabulary app state and VocabularyManager instance

**Structure**:
```rust
pub struct VocabularyState {
    manager: VocabularyManager<EmbeddedStorage>,
    current_card_id: Option<CardId>,
    review_mode: ReviewMode,
}

impl VocabularyState {
    pub fn new() -> Result<Self, VocabularyError>;
    pub fn manager(&self) -> &VocabularyManager<EmbeddedStorage>;
    pub fn manager_mut(&mut self) -> &mut VocabularyManager<EmbeddedStorage>;
}
```

#### Step 3: Create Learning View Components

##### 3.1 Vocabulary Menu View
**File**: `cardworder/src/logic/views/vocabulary_menu.rs`

**Purpose**: Main menu for vocabulary learning features

**Menu Options**:
- Review (start quiz session)
- Add Word
- List Words
- Statistics
- Back (return to main menu)

##### 3.2 Review Session View
**File**: `cardworder/src/logic/views/review_session.rs`

**Purpose**: Interactive quiz interface for spaced repetition

**Flow**:
1. Get next due card from VocabularyManager
2. Display word (front side)
3. Wait for user input (show answer)
4. Display rating buttons (Again, Hard, Good, Easy)
5. Record rating with FSRS
6. Navigate to next card or back to menu

**UI Elements**:
- Word display (front/back)
- Status bar (progress, cards remaining)
- Rating buttons (4-way navigation)
- Keyboard shortcuts for ratings

##### 3.3 Add Word View
**File**: `cardworder/src/logic/views/add_word.rs`

**Purpose**: Form to add new words to vocabulary

**Flow**:
1. Display input form (front/back text fields)
2. Accept text input via keyboard
3. Save new word card
4. Return to vocabulary menu

**UI Elements**:
- Text input fields (front/back)
- Add button
- Cancel button

##### 3.4 List Words View
**File**: `cardworder/src/logic/views/list_words.rs`

**Purpose**: Display all words with review status

**Features**:
- Scrollable list of words
- Show due/not due status
- Display review count
- Show next review date

##### 3.5 Statistics View  
**File**: `cardworder/src/logic/views/vocabulary_stats.rs`

**Purpose**: Display learning statistics

**Metrics**:
- Total words
- Words due for review
- Total reviews
- Success rate

#### Step 4: Integrate with Existing View System

##### 4.1 Update View Manager
**File**: `cardworder/src/logic/view_manager.rs`

**Changes**:
- Add `VocabularyState` to app state
- Add vocabulary view variants to `CardputerView` enum
- Handle vocabulary view transitions

##### 4.2 Update Main Menu
**File**: `cardworder/src/logic/views/main_menu.rs`

**Changes**:
- Add "Vocabulary Learning" menu option
- Handle transition to vocabulary menu
- Pass vocabulary state to vocabulary views

#### Step 5: Testing Strategy

##### Unit Tests
- Storage implementation tests (mock data)
- VocabularyManager integration tests
- View state transition tests

##### Integration Tests
- End-to-end review session
- Add word → Review → Complete session
- Multiple cards with different due dates

##### Hardware Tests
- SD card read/write operations
- Memory usage monitoring
- Performance profiling (review session speed)

### Implementation Checklist

- [ ] Create `EmbeddedStorage` struct implementing `Storage` trait
- [ ] Implement SD card file I/O operations
- [ ] Add serialization/deserialization for WordCard
- [ ] Create `VocabularyState` wrapper for VocabularyManager
- [ ] Implement `VocabularyMenuView` with navigation
- [ ] Implement `ReviewSessionView` with quiz interface
- [ ] Implement `AddWordView` with input form
- [ ] Implement `ListWordsView` with scrollable list
- [ ] Implement `VocabularyStatsView` with metrics
- [ ] Update `ViewManager` to support vocabulary views
- [ ] Update `MainMenuView` to include vocabulary option
- [ ] Test storage operations on embedded platform
- [ ] Test review session workflow
- [ ] Test add word workflow
- [ ] Test integration with existing UI system
- [ ] Performance testing (memory usage, response time)
- [ ] Hardware validation on Cardputer device

### Integration Architecture Flow

```
┌─────────────────────────────────────────────────────────────┐
│                    CARDWORDER EMBEDDED APP                   │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  ┌──────────────┐    ┌──────────────┐    ┌──────────────┐ │
│  │ Main Menu    │───▶│ Vocabulary   │───▶│ Review      │ │
│  │ View         │    │ Menu View    │    │ Session View│ │
│  └──────────────┘    └──────────────┘    └──────────────┘ │
│         │                   │                    │          │
│         │                   ▼                    │          │
│         │         ┌──────────────┐               │          │
│         │         │ Vocabulary   │               │          │
│         │         │ State        │◀──────────────┘          │
│         │         └──────────────┘                          │
│         │                   │                              │
│         │                   ▼                              │
│         │         ┌──────────────┐                         │
│         └────────▶│ Vocabulary   │                         │
│                   │ Manager      │                         │
│                   └──────────────┘                         │
│                            │                               │
│                            ▼                               │
│                   ┌──────────────┐                        │
│                   │ Embedded      │                        │
│                   │ Storage       │                        │
│                   └──────────────┘                        │
│                            │                               │
│                            ▼                               │
│                   ┌──────────────┐                        │
│                   │ SD Card /    │                        │
│                   │ Flash Storage│                        │
│                   └──────────────┘                        │
└─────────────────────────────────────────────────────────────┘

Main Components:
1. Views: UI layer (VocabularyMenu, ReviewSession, AddWord, etc.)
2. VocabularyState: App state wrapper around VocabularyManager
3. VocabularyManager: Core FSRS learning logic (from fsrs-core)
4. EmbeddedStorage: Platform-specific storage implementation
5. Storage Backend: SD card or flash storage
```

### Key Design Decisions

#### 1. Storage Strategy
**Decision**: Use in-memory `heapless::Vec` with periodic SD card sync
**Rationale**: 
- Faster operations during review sessions
- Reduced SD card wear
- Simpler error handling (RAM is more reliable than SD card)
- Can batch writes for efficiency

#### 2. View Integration Pattern  
**Decision**: Add vocabulary views to existing view enum, pass state as context
**Rationale**:
- Maintains existing view manager pattern
- Clean separation between UI and business logic
- Easy to add/remove vocabulary features

#### 3. Input Handling
**Decision**: Reuse existing keyboard input system with custom handlers for learning views
**Rationale**:
- Consistent with current UI patterns
- Leverage existing input abstractions
- Support for keyboard shortcuts (1=Again, 2=Hard, 3=Good, 4=Easy)

#### 4. Memory Management
**Decision**: Limit to fixed number of cards (e.g., 100 cards max)
**Rationale**:
- Embedded memory constraints
- Predictable memory usage
- `heapless::Vec` with capacity provides compile-time guarantees

### Dependencies & Constraints

#### Embedding fsrs-core Library
- **Dependency**: `fsrs-core = { path = "../fsrs-core" }` (already in Cargo.toml)
- **No-std**: fsrs-core is no_std compatible
- **Allocations**: Use `heapless` collections to minimize heap allocations

#### Storage Requirements
- **File Format**: JSON for human readability
- **Location**: `/fs/vocabulary.json` on SD card
- **Size Limit**: ~64KB per file (accommodates ~100 cards)

#### Platform Integration
- **ESP-IDF**: Use `esp-idf-hal` for SD card access
- **embassy**: Could use embassy timers for FSRS scheduling if needed
- **FreeRTOS**: Current task management is sufficient

### Testing Strategy

#### Unit Testing
- **Storage Mock**: Create in-memory mock storage for testing
- **FSRS Algorithm**: Already tested in fsrs-core (39 tests passing)
- **View Components**: Test view state transitions

#### Integration Testing  
- **Review Workflow**: Add 3 words → Review all → Verify scheduling
- **Storage Persistence**: Save words → Reboot → Verify data loaded
- **Multiple Sessions**: Test FSRS state across multiple review sessions

#### Performance Testing
- **Memory Usage**: Profile heap usage with different card counts
- **SD Card Speed**: Measure read/write performance
- **UI Responsiveness**: Ensure smooth review flow (<100ms response time)

### Challenges & Mitigations

#### Challenge 1: SD Card Reliability
**Problem**: SD cards can be removed, corrupted, or slow to access
**Mitigation**: 
- Use RAM buffer with periodic sync
- Add error recovery (graceful degradation if SD fails)
- Provide "Save" button as backup

#### Challenge 2: Limited Screen Space
**Problem**: Cardputer screen is small (128x128 or similar)
**Mitigation**:
- Use existing scrollable form system
- Compact UI with essential information only
- Keyboard shortcuts for common actions

#### Challenge 3: Text Input on Embedded
**Problem**: Text input is more complex than on desktop
**Mitigation**:
- Use existing input text support (InputText UiLineElement)
- Keyboard-based input with visual cursor
- Support for multiple languages (already in system)

#### Challenge 4: FSRS Time Handling
**Problem**: Embedded devices may not have accurate time
**Mitigation**:
- Support manual time input for testing
- Use NTP sync when available (already in main menu)
- Use relative timestamps for due date calculations

## Creative Phases Required ✅ COMPLETE

### 🎨 Architecture Design ✅
- **Decision**: Selected simple trait-based storage abstraction with associated types
- **Rationale**: Best balance of simplicity and flexibility for cross-platform support
- **Component**: `memory-bank/creative/creative-storage-abstraction.md`

### 🏗️ Data Model Design ✅
- **Decision**: Selected owned String wrapper with direct rs-fsrs Card integration
- **Rationale**: Best serialization compatibility and simplest integration approach
- **Component**: `memory-bank/creative/creative-data-model.md`

### ⚙️ API Design ✅
- **Decision**: Selected Manager Pattern with VocabularyManager as entry point
- **Rationale**: Simple, ergonomic API with clear ownership model for both platforms
- **Component**: `memory-bank/creative/creative-api-design.md`

## Challenges & Mitigations

### Challenge 1: Cross-Platform Compatibility
**Problem**: Console and embedded platforms have different constraints (std vs no_std)
**Mitigation**: Use conditional compilation and trait-based design that works for both

### Challenge 2: Storage Abstraction
**Problem**: Different storage backends (JSON file vs embedded flash/SD)
**Mitigation**: Define flexible storage traits that can be implemented for any backend

### Challenge 3: Workspace Management
**Problem**: Managing multiple crates in a workspace can be complex
**Mitigation**: Use Cargo workspace features and shared dependencies

### Challenge 4: Integration with Existing Codebase
**Problem**: Need to integrate FSRS into existing Cardputer UI without breaking changes
**Mitigation**: Create library as separate module, integrate incrementally

## Technology Validation Checkpoints ✅
- [x] rs-fsrs v1.2.1 dependency verified in Cargo.toml
- [x] Build environment validated (Rust 1.82.0, Cargo 1.82.0)
- [x] Storage trait interface designed (with implementation notes)
- [x] Console UI approach validated (std::io, no external dependencies)
- [x] Cross-platform support confirmed

## Testing Strategy
- [x] Unit tests for fsrs-core models ✅ (13 tests)
- [x] Unit tests for FSRS algorithm logic ✅ (12 tests)
- [x] Unit tests for vocabulary manager ✅ (14 tests)
- [ ] Integration tests for console app
- [ ] Platform-specific tests for embedded app
- [ ] End-to-end learning session tests

## Documentation Plan
- [ ] fsrs-core library API documentation
- [ ] Storage trait usage guide
- [ ] Console app user documentation
- [ ] Integration guide for embedded developers
- [ ] Architecture decision records

## Dependencies
- rs-fsrs v1.2.1 (existing)
- serde, serde_json (existing)
- chrono (new for console app)
- anyhow (new for error handling in console)

## Current Status
- Phase: BUILD MODE - Planning for Phase 5 (Embedded Integration)
- Status: Plan Complete
- Blockers: None
- Next Step: Begin implementation of embedded storage and view components

## Archive
- **Date**: 2025-10-29
- **Archive Document**: [memory-bank/archive/archive-scrollable-ui-20251029.md](memory-bank/archive/archive-scrollable-ui-20251029.md)
- **Reflection Document**: [memory-bank/reflection/reflection-scrollable-ui.md](memory-bank/reflection/reflection-scrollable-ui.md)
- **Status**: COMPLETED

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

---

## BUILD Phase Summary

### Workspace Structure Created
- ✅ Converted project to Cargo workspace
- ✅ Created workspace root `Cargo.toml`
- ✅ Set up three crates: `cardworder`, `fsrs-core`, `console-learner`
- ✅ All crates compile successfully

### fsrs-core Library ✅
**Location**: `fsrs-core/`

**Files Created**:
- `fsrs-core/Cargo.toml` - Library dependencies
- `fsrs-core/src/lib.rs` - Public API exports
- `fsrs-core/src/models.rs` - WordCard data structures with Card integration
- `fsrs-core/src/storage.rs` - Storage trait definition
- `fsrs-core/src/vocabulary.rs` - VocabularyManager implementation
- `fsrs-core/src/fsrs.rs` - FSRS scheduling logic (simplified)
- `fsrs-core/src/error.rs` - Error types

**Features Implemented**:
- Trait-based storage abstraction
- WordCard model with FSRS Card integration
- VocabularyManager API for card management
- Rating enum (Again, Hard, Good, Easy)
- Basic FSRS scheduling (simplified implementation)
- Due card filtering
- Statistics calculation

### console-learner Application ✅
**Location**: `console-learner/`

**Files Created**:
- `console-learner/Cargo.toml` - Console app dependencies
- `console-learner/src/main.rs` - Entry point
- `console-learner/src/app.rs` - Main application logic
- `console-learner/src/storage.rs` - JSON file storage implementation
- `console-learner/src/ui/mod.rs` - UI module exports
- `console-learner/src/ui/menu.rs` - Menu display and input helpers
- `console-learner/src/ui/review.rs` - Review session workflow

**Features Implemented**:
- Text-based menu system
- Add word flow with prompts
- Review session with interactive quiz
- List all words functionality
- Statistics display
- JSON file persistence
- Platform-specific screen clearing

### Current Build Status
- ✅ `fsrs-core` compiles successfully
- ✅ `console-learner` compiles successfully
- ✅ `cardworder` maintains compatibility
- ✅ Unit tests passing (39 tests total)
- ⚠️ Phase 5 (Embedded Integration) not yet implemented

### Test Summary
**Total Tests: 39 (All Passing)**

#### Module Test Breakdown:
- **error.rs**: 4 tests - Error creation, formatting, Error trait implementation, comparison
- **models.rs**: 8 tests - WordCard creation, examples, reviews, serialization, legacy format, FSRS state
- **fsrs.rs**: 12 tests - Rating conversion, card updates with different ratings, due date checks, stability progression
- **vocabulary.rs**: 14 tests - VocabularyManager operations (add, review, delete, stats), multiple reviews, error handling
- **storage.rs**: (Mock implementation used in vocabulary tests)

### Next Steps
- Test console-learner application
- Implement Phase 5: Embedded integration  
- Add integration tests for console app
- Add platform-specific tests for embedded app
