# Creative Phase: API Design

🎨🎨🎨 ENTERING CREATIVE PHASE: API DESIGN 🎨🎨🎨

## Focus: Public API Surface for fsrs-core Library
## Objective: Design clean, ergonomic API for console and embedded consumers
## Requirements: Type-safe, platform-agnostic, easy to use

---

## 1️⃣ PROBLEM

**Description**: Design the public API for fsrs-core library that will be used by both console app and embedded Cardputer app.

**Requirements**:
- Single entry point (Library struct or module)
- Hide storage implementation details
- Type-safe error handling
- Platform-agnostic interface
- Support for all FSRS operations (review, schedule, query)
- Easy to integrate into existing codebases

**Constraints**:
- Must work in std and no_std environments
- Minimize allocations for embedded
- Clear separation between library and storage
- Export only necessary types/functions

---

## 2️⃣ OPTIONS

### Option A: Manager Pattern
Single `VocabularyManager` struct that coordinates storage and FSRS logic.

### Option B: Builder Pattern
Builder for creating manager instances with configuration.

### Option C: Module-Based API
Separate modules for storage, FSRS, and cards with re-exports.

---

## 3️⃣ ANALYSIS

| Criterion | Option A: Manager | Option B: Builder | Option C: Modules |
|-----------|-------------------|-------------------|-------------------|
| **Simplicity** | ⭐⭐⭐⭐ | ⭐⭐⭐ | ⭐⭐⭐⭐ |
| **Ergonomics** | ⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ | ⭐⭐⭐ |
| **Flexibility** | ⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ |
| **Discoverability** | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐ | ⭐⭐⭐ |
| **Memory Usage** | ⭐⭐⭐⭐ | ⭐⭐⭐ | ⭐⭐⭐⭐ |
| **Embedded Support** | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ |

**Key Insights**:
- Option A provides single entry point with clear ownership
- Option B offers most flexible configuration at creation time
- Option C provides modular access but less ergonomic

---

## 4️⃣ DECISION

**Selected**: Option A: Manager Pattern with Optional Builder

**Rationale**:
- Simple, single entry point (`VocabularyManager`)
- Clear ownership and lifetime management
- Works well for both console and embedded
- Can add builder helper for complex initialization if needed
- Consumers only need to import one type to get started
- Easy to document and maintain

**API Structure**:
```rust
pub struct VocabularyManager<S: Storage> {
    storage: S,
    fsrs: FSRS,
}

impl<S: Storage> VocabularyManager<S> {
    pub fn new(storage: S) -> Self;
    pub fn add_card(&mut self, word: WordCard) -> Result<CardId, Error>;
    pub fn review_card(&mut self, id: CardId, rating: Rating) -> Result<(), Error>;
    pub fn get_due_cards(&self) -> Result<Vec<WordCard>, Error>;
    pub fn get_all_cards(&self) -> Result<Vec<WordCard>, Error>;
}
```

---

## 5️⃣ IMPLEMENTATION GUIDELINES

### Public API Exports (lib.rs)
```rust
pub mod models;
pub mod storage;
pub mod fsrs;

pub use models::WordCard;
pub use storage::Storage;
pub use vocabulary::VocabularyManager;
```

### Core Components
- **VocabularyManager**: Main entry point
  - Owns storage and FSRS scheduler
  - Methods for add, review, query operations
  - Lifetime-aware for embedded constraints
  
- **Storage Trait**: Abstract storage interface
  - Generic over storage backend
  - Provides save/load/delete operations
  
- **Models Module**: Data types
  - WordCard, CardId types
  - Helper functions for card operations
  
- **FSRS Module**: Algorithm integration
  - Wrapper around rs-fsrs library
  - Rating types and scheduling logic

### Error Handling
- `VocabularyError`: Main error type
- Enum variants for different failure modes
- Storage-specific errors propagate
- Clear error messages for debugging

### Console App Usage
```rust
use fsrs_core::VocabularyManager;
use fsrs_core::storage::JsonFileStorage;

let storage = JsonFileStorage::new("words.json")?;
let mut manager = VocabularyManager::new(storage);
manager.add_card(word)?;
let due_cards = manager.get_due_cards()?;
```

### Embedded App Usage
```rust
use fsrs_core::VocabularyManager;
use cardworder::storage::SdCardStorage;

let storage = SdCardStorage::new()?;
let mut manager = VocabularyManager::new(storage);
// Same API as console app
```

### Cross-Platform Support
- Feature gates for std vs no_std
- Conditional compilation for platform-specific storage
- Shared core API across platforms
- No platform-specific code in public API

---

🎨🎨🎨 EXITING CREATIVE PHASE 🎨🎨🎨

**Summary**: Selected Manager Pattern with clean public API
**Key Decisions**: VocabularyManager entry point, generic Storage trait, error handling strategy
**Next Steps**: Implement lib.rs exports and VocabularyManager in fsrs-core

