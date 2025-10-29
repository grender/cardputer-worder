# Creative Phase: Data Model Design

🎨🎨🎨 ENTERING CREATIVE PHASE: DATA MODEL 🎨🎨🎨

## Focus: Word Card and Data Structure Design
## Objective: Design serializable data models for vocabulary learning
## Requirements: Compatible with rs-fsrs library, serializable, cross-platform

---

## 1️⃣ PROBLEM

**Description**: Design data structures for vocabulary learning with FSRS scheduling that can be serialized and work on both console and embedded platforms.

**Requirements**:
- Compatible with rs-fsrs library (Card type)
- Serializable with serde (JSON)
- Contains word data (front, back, examples)
- Stores FSRS state (stability, difficulty, due date)
- Compact memory footprint for embedded
- Human-readable JSON for debugging

**Constraints**:
- Must integrate with rs-fsrs v1.2.1
- JSON size matters for embedded (flash memory limits)
- Preserve FSRS algorithm state across sessions
- Support batch operations (save multiple cards)

---

## 2️⃣ OPTIONS

### Option A: Wrapper with Owned Strings
Embedded WordCard structure with owned String fields, wraps rs-fsrs Card.

### Option B: References and Lifetimes
Use references to minimize copying, borrow checker constraints.

### Option C: Compact Binary + JSON Metadata
Hybrid approach: binary for FSRS state, JSON for word data.

---

## 3️⃣ ANALYSIS

| Criterion | Option A: Owned Strings | Option B: References | Option C: Hybrid |
|-----------|-------------------------|----------------------|------------------|
| **Simplicity** | ⭐⭐⭐⭐ | ⭐⭐ | ⭐⭐ |
| **Serialization** | ⭐⭐⭐⭐⭐ | ⭐⭐ | ⭐⭐⭐⭐ |
| **Memory Usage** | ⭐⭐⭐ | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ |
| **Platform Support** | ⭐⭐⭐⭐⭐ | ⭐⭐⭐ | ⭐⭐⭐⭐⭐ |
| **Maintainability** | ⭐⭐⭐⭐⭐ | ⭐⭐⭐ | ⭐⭐⭐ |
| **FSRS Integration** | ⭐⭐⭐⭐⭐ | ⭐⭐⭐ | ⭐⭐⭐ |

**Key Insights**:
- Option A offers best serialization experience and platform compatibility
- Option B minimizes allocations but complicates borrowing
- Option C offers best memory efficiency but adds complexity

---

## 4️⃣ DECISION

**Selected**: Option A: Wrapper with Owned Strings

**Rationale**:
- Simplest integration with rs-fsrs library
- Best serde compatibility for JSON serialization
- Easy to maintain and understand
- String overhead acceptable for vocabulary learning (typically 10-100 words)
- Can optimize later with string pooling if needed
- Works seamlessly on both std and no_std with alloc support

**Data Model Structure**:
```rust
pub struct WordCard {
    id: CardId,              // Unique identifier
    front: String,            // English word
    back: String,            // Translation/definition
    examples: Vec<String>,   // Example sentences
    card: Card,              // rs-fsrs card state
    created_at: i64,         // Timestamp
    last_review: Option<i64>, // Last review timestamp
}
```

**Card State**:
- Direct integration with rs-fsrs `Card` type
- Store FSRS algorithm parameters (stability, difficulty, retrievability)
- Support review scheduling
- Track learning progress

---

## 5️⃣ IMPLEMENTATION GUIDELINES

### Core Data Structures
```rust
use rs_fsrs::Card;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WordCard {
    pub id: CardId,
    pub front: String,
    pub back: String,
    pub examples: Vec<String>,
    pub card_state: Card,
    pub created_at: i64,
    pub last_review: Option<i64>,
}

pub type CardId = u64;
```

### fsrs-core Models
- `models.rs`: Core data structures
- `CardId`: Unique identifier type
- `WordCard`: Main card structure
- Helper functions for card operations

### Serialization Strategy
- Use default serde_json serialization
- Compact JSON format (no pretty printing for embedded)
- Include all FSRS state in serialized data
- Support loading empty collections

### Integration Points
- rs-fsrs `Card` type stored directly
- Convert to/from JSON for storage
- Track timestamps for review scheduling
- Support batch save/load operations

---

🎨🎨🎨 EXITING CREATIVE PHASE 🎨🎨🎨

**Summary**: Selected owned String wrapper approach with direct rs-fsrs Card integration
**Key Decisions**: WordCard structure, CardId type, JSON serialization strategy
**Next Steps**: Implement `models.rs` in fsrs-core with serialization support

