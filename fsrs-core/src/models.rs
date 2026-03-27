use rs_fsrs::Card;
use serde::{Deserialize, Serialize};

pub type WordPairId = u64;

/// Direction of review for a word pair
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Direction {
    /// Show English, recall Russian
    Forward = 0,
    /// Show Russian, recall English
    Reverse = 1,
}

/// FSRS scheduling state for one direction of a word pair
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DirectionState {
    pub card: Card,
    pub last_review: Option<String>,
}

impl DirectionState {
    pub fn new() -> Self {
        Self {
            card: Card::new(),
            last_review: None,
        }
    }
}

/// A word pair with two independent FSRS scheduling states
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WordPair {
    pub id: WordPairId,
    pub en: String,
    pub ru: String,
    pub examples: Vec<String>,
    /// FSRS state: index 0 = Forward (EN→RU), index 1 = Reverse (RU→EN)
    pub fsrs: [DirectionState; 2],
    pub created_at: String,
}

impl WordPair {
    pub fn new(id: WordPairId, en: String, ru: String) -> Self {
        Self {
            id,
            en,
            ru,
            examples: Vec::new(),
            fsrs: [DirectionState::new(), DirectionState::new()],
            created_at: get_timestamp_iso(),
        }
    }

    pub fn add_example(&mut self, example: String) {
        self.examples.push(example);
    }

    /// Get the FSRS state for a given direction
    pub fn direction_state(&self, dir: Direction) -> &DirectionState {
        &self.fsrs[dir as usize]
    }

    /// Get mutable FSRS state for a given direction
    pub fn direction_state_mut(&mut self, dir: Direction) -> &mut DirectionState {
        &mut self.fsrs[dir as usize]
    }

    /// Build a ReviewItem for the given direction
    pub fn review_item(&self, direction: Direction) -> ReviewItem {
        let (prompt, answer) = match direction {
            Direction::Forward => (self.en.clone(), self.ru.clone()),
            Direction::Reverse => (self.ru.clone(), self.en.clone()),
        };
        ReviewItem {
            pair_id: self.id,
            direction,
            prompt,
            answer,
            examples: self.examples.clone(),
            card: self.fsrs[direction as usize].card.clone(),
        }
    }
}

/// Computed view for the review UI (not persisted)
#[derive(Debug, Clone)]
pub struct ReviewItem {
    pub pair_id: WordPairId,
    pub direction: Direction,
    pub prompt: String,
    pub answer: String,
    pub examples: Vec<String>,
    pub card: Card,
}

/// File-level container for serialization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PairsFile {
    pub next_id: WordPairId,
    pub pairs: Vec<WordPair>,
}

#[cfg(feature = "std")]
pub fn get_timestamp_iso() -> String {
    chrono::Utc::now().to_rfc3339()
}

#[cfg(not(feature = "std"))]
pub fn get_timestamp_iso() -> String {
    "1970-01-01T00:00:00Z".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_word_pair_new() {
        let pair = WordPair::new(1, "bat".to_string(), "летучая мышь".to_string());

        assert_eq!(pair.id, 1);
        assert_eq!(pair.en, "bat");
        assert_eq!(pair.ru, "летучая мышь");
        assert!(pair.examples.is_empty());
        assert!(!pair.created_at.is_empty());
        assert!(pair.fsrs[0].last_review.is_none());
        assert!(pair.fsrs[1].last_review.is_none());
        assert_eq!(pair.fsrs[0].card.reps, 0);
        assert_eq!(pair.fsrs[1].card.reps, 0);
    }

    #[test]
    fn test_review_item_forward() {
        let pair = WordPair::new(1, "bat".to_string(), "летучая мышь".to_string());
        let item = pair.review_item(Direction::Forward);

        assert_eq!(item.pair_id, 1);
        assert_eq!(item.direction, Direction::Forward);
        assert_eq!(item.prompt, "bat");
        assert_eq!(item.answer, "летучая мышь");
    }

    #[test]
    fn test_review_item_reverse() {
        let pair = WordPair::new(1, "bat".to_string(), "летучая мышь".to_string());
        let item = pair.review_item(Direction::Reverse);

        assert_eq!(item.pair_id, 1);
        assert_eq!(item.direction, Direction::Reverse);
        assert_eq!(item.prompt, "летучая мышь");
        assert_eq!(item.answer, "bat");
    }

    #[test]
    fn test_postcard_serialization_roundtrip() {
        let mut pair = WordPair::new(1, "hello".to_string(), "привет".to_string());
        pair.add_example("Hello, world!".to_string());

        let file = PairsFile {
            next_id: 2,
            pairs: vec![pair],
        };

        let bytes = postcard::to_allocvec(&file).unwrap();
        let decoded: PairsFile = postcard::from_bytes(&bytes).unwrap();

        assert_eq!(decoded.next_id, 2);
        assert_eq!(decoded.pairs.len(), 1);
        assert_eq!(decoded.pairs[0].en, "hello");
        assert_eq!(decoded.pairs[0].ru, "привет");
        assert_eq!(decoded.pairs[0].examples, vec!["Hello, world!"]);
    }

    #[test]
    fn test_json_serialization_roundtrip() {
        let pair = WordPair::new(1, "cat".to_string(), "кот".to_string());
        let file = PairsFile {
            next_id: 2,
            pairs: vec![pair],
        };

        let json = serde_json::to_string(&file).unwrap();
        let decoded: PairsFile = serde_json::from_str(&json).unwrap();

        assert_eq!(decoded.pairs[0].en, "cat");
        assert_eq!(decoded.pairs[0].ru, "кот");
    }

    #[test]
    fn test_direction_state_independence() {
        let mut pair = WordPair::new(1, "dog".to_string(), "собака".to_string());

        pair.direction_state_mut(Direction::Forward).last_review =
            Some("2026-01-01T00:00:00Z".to_string());

        assert!(pair.direction_state(Direction::Forward).last_review.is_some());
        assert!(pair.direction_state(Direction::Reverse).last_review.is_none());
    }
}
