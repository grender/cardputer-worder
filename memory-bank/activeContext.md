# Active Context: CardWorder Development

## Recently Completed

### Scrollable Minimalist UI (2025-10-29)
**Status**: ✅ COMPLETED

The foundational UI infrastructure for CardWorder has been successfully implemented and archived. This includes:
- Modular scrollable form system with `fonts.rs` and `render.rs`
- Visual scroll bar with proportional sizing and spatial awareness
- Text input support with cursor positioning
- Efficient viewport-based rendering for embedded constraints
- 12-item menu demonstration of scrolling capabilities

**Archive**: [memory-bank/archive/archive-scrollable-ui-20251029.md](memory-bank/archive/archive-scrollable-ui-20251029.md)

---

## Current Focus

### New Task: Vocabulary Learning System with FSRS Library

**Status**: PLAN Mode - Comprehensive Planning Complete

**Requirements:**
1. Console application for learning English words
2. Common library for FSRS functionality  
3. Storage abstraction layer
4. Cross-platform support (console + embedded)

**Complexity Level**: LEVEL 3 - Intermediate Feature

**Implementation Plan**: ✅ Created
- Architecture: 3-component system (fsrs-core library, console app, embedded integration)
- Technology: Cargo workspace, rs-fsrs v1.2.1, trait-based storage abstraction
- Phases: 5 implementation phases from setup to embedded integration

**Creative Phase**: ✅ COMPLETE
- Architecture: Simple trait-based storage abstraction selected
- Data Model: Owned String wrapper with rs-fsrs integration selected
- API Design: VocabularyManager pattern selected
- Documents: All 3 creative phase docs created in `memory-bank/creative/`

**Next Step**: **VAN QA MODE** for technology validation before implementation

---

## Context for Next Task

### Technical Environment
- **Platform**: ESP32 (Cardputer)
- **Programming Language**: Rust (2021 edition)
- **UI Framework**: Embedded-graphics with custom Cardputer UI
- **Key Dependencies**: 
  - esp-idf-svc, esp-idf-hal for hardware
  - rs-fsrs for spaced repetition algorithm
  - serde, serde_json for data serialization

### Available UI Infrastructure
- ✅ Scrollable forms with dynamic composing
- ✅ Visual scroll bar for spatial awareness
- ✅ Text input support with cursor positioning
- ✅ Font measurement and multiline text wrapping
- ✅ Efficient viewport rendering for embedded constraints

### Project Goals
- Help users learn English words using spaced repetition
- Provide intuitive interface for word learning and review
- Ensure reliable time updates via NTP for correct repetition intervals
- Leverage device hardware (screen, keyboard, WiFi, speaker)

### Development Guidelines
- No high-level UI abstractions (manual construction only)
- Explicit vertical stacking as primary layout
- Monospace fonts for simplified measurement
- Defensive programming for ESP32 constraints
- Performance optimization for low memory usage

---

## Memory Bank Status

- **tasks.md**: ✅ Updated with completion status
- **progress.md**: ✅ Created with milestone tracking
- **activeContext.md**: ✅ Created with current focus
- **systemPatterns.md**: ✅ Contains monospace word wrapping pattern
- **techContext.md**: ✅ Contains platform constraints
- **productContext.md**: ✅ Contains UI/UX constraints

---

## Suggested Next Action

**VAN MODE**: Use VAN mode to initialize the next development task. The UI foundation is complete and ready for core functionality implementation.

Potential task types:
- **Level 2**: Simple enhancement (e.g., word data structures)
- **Level 3**: Intermediate feature (e.g., vocabulary management screen with FSRS)
- **Level 4**: Complex system (e.g., complete learning workflow)

