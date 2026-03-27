use crate::models::{WordPairId, WordPair, Direction, ReviewItem, get_timestamp_iso};
use crate::storage::PairStorage;
use crate::error::VocabularyError;
use crate::fsrs::{Rating, update_card_with_review, is_card_due};

pub struct VocabularyManager<S: PairStorage> {
    storage: S,
    next_pair_id: WordPairId,
}

impl<S: PairStorage> VocabularyManager<S> {
    pub fn new(storage: S) -> Self {
        Self {
            storage,
            next_pair_id: 1,
        }
    }

    pub fn with_next_id(storage: S, next_id: WordPairId) -> Self {
        Self {
            storage,
            next_pair_id: next_id,
        }
    }

    pub fn add_pair(&mut self, en: String, ru: String) -> Result<WordPairId, VocabularyError> {
        let id = self.next_pair_id;
        self.next_pair_id += 1;
        let pair = WordPair::new(id, en, ru);
        self.storage.save_pair(pair)?;
        Ok(id)
    }

    pub fn review(
        &mut self,
        pair_id: WordPairId,
        direction: Direction,
        rating: Rating,
    ) -> Result<(), VocabularyError> {
        let mut pair = self.storage.load_pair(pair_id)?;
        let state = pair.direction_state_mut(direction);
        update_card_with_review(&mut state.card, rating)?;
        state.last_review = Some(get_timestamp_iso());
        self.storage.save_pair(pair)?;
        Ok(())
    }

    pub fn get_due_reviews(&self) -> Result<Vec<ReviewItem>, VocabularyError> {
        let all_pairs = self.storage.load_all_pairs()?;
        let mut due = Vec::new();
        for pair in &all_pairs {
            for dir in [Direction::Forward, Direction::Reverse] {
                if is_card_due(&pair.direction_state(dir).card) {
                    due.push(pair.review_item(dir));
                }
            }
        }
        Ok(due)
    }

    pub fn get_all_pairs(&self) -> Result<Vec<WordPair>, VocabularyError> {
        self.storage.load_all_pairs()
    }

    pub fn get_pair(&self, id: WordPairId) -> Result<WordPair, VocabularyError> {
        self.storage.load_pair(id)
    }

    pub fn delete_pair(&mut self, id: WordPairId) -> Result<(), VocabularyError> {
        self.storage.delete_pair(id)
    }

    pub fn get_stats(&self) -> Result<Stats, VocabularyError> {
        let all_pairs = self.storage.load_all_pairs()?;
        let total_pairs = all_pairs.len();
        let mut due_forward = 0;
        let mut due_reverse = 0;
        for pair in &all_pairs {
            if is_card_due(&pair.direction_state(Direction::Forward).card) {
                due_forward += 1;
            }
            if is_card_due(&pair.direction_state(Direction::Reverse).card) {
                due_reverse += 1;
            }
        }
        Ok(Stats {
            total_pairs,
            due_forward,
            due_reverse,
            due_total: due_forward + due_reverse,
        })
    }

    pub fn next_pair_id(&self) -> WordPairId {
        self.next_pair_id
    }

    pub fn storage_mut(&mut self) -> &mut S {
        &mut self.storage
    }
}

#[derive(Debug, Clone)]
pub struct Stats {
    pub total_pairs: usize,
    pub due_forward: usize,
    pub due_reverse: usize,
    pub due_total: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::WordPair;
    use crate::VocabularyError;
    use std::collections::HashMap;

    struct MockStorage {
        pairs: HashMap<WordPairId, WordPair>,
    }

    impl MockStorage {
        fn new() -> Self {
            Self { pairs: HashMap::new() }
        }
    }

    impl PairStorage for MockStorage {
        fn save_pair(&mut self, pair: WordPair) -> Result<(), VocabularyError> {
            self.pairs.insert(pair.id, pair);
            Ok(())
        }

        fn load_pair(&self, id: WordPairId) -> Result<WordPair, VocabularyError> {
            self.pairs.get(&id).cloned().ok_or(VocabularyError::CardNotFound(id))
        }

        fn load_all_pairs(&self) -> Result<Vec<WordPair>, VocabularyError> {
            Ok(self.pairs.values().cloned().collect())
        }

        fn delete_pair(&mut self, id: WordPairId) -> Result<(), VocabularyError> {
            self.pairs.remove(&id).ok_or(VocabularyError::CardNotFound(id)).map(|_| ())
        }
    }

    #[test]
    fn test_add_pair() {
        let mut manager = VocabularyManager::new(MockStorage::new());
        let id = manager.add_pair("bat".to_string(), "летучая мышь".to_string()).unwrap();
        assert_eq!(id, 1);

        let pair = manager.get_pair(id).unwrap();
        assert_eq!(pair.en, "bat");
        assert_eq!(pair.ru, "летучая мышь");
    }

    #[test]
    fn test_review_forward() {
        let mut manager = VocabularyManager::new(MockStorage::new());
        let id = manager.add_pair("cat".to_string(), "кот".to_string()).unwrap();

        manager.review(id, Direction::Forward, Rating::Good).unwrap();

        let pair = manager.get_pair(id).unwrap();
        assert_eq!(pair.fsrs[0].card.reps, 1);
        assert!(pair.fsrs[0].last_review.is_some());
        // Reverse direction unchanged
        assert_eq!(pair.fsrs[1].card.reps, 0);
        assert!(pair.fsrs[1].last_review.is_none());
    }

    #[test]
    fn test_review_reverse() {
        let mut manager = VocabularyManager::new(MockStorage::new());
        let id = manager.add_pair("dog".to_string(), "собака".to_string()).unwrap();

        manager.review(id, Direction::Reverse, Rating::Easy).unwrap();

        let pair = manager.get_pair(id).unwrap();
        assert_eq!(pair.fsrs[1].card.reps, 1);
        assert!(pair.fsrs[1].last_review.is_some());
        // Forward direction unchanged
        assert_eq!(pair.fsrs[0].card.reps, 0);
    }

    #[test]
    fn test_get_due_reviews() {
        let mut manager = VocabularyManager::new(MockStorage::new());
        manager.add_pair("hello".to_string(), "привет".to_string()).unwrap();

        let due = manager.get_due_reviews().unwrap();
        // New cards are due in both directions
        assert_eq!(due.len(), 2);

        let forward = due.iter().find(|r| r.direction == Direction::Forward).unwrap();
        assert_eq!(forward.prompt, "hello");
        assert_eq!(forward.answer, "привет");

        let reverse = due.iter().find(|r| r.direction == Direction::Reverse).unwrap();
        assert_eq!(reverse.prompt, "привет");
        assert_eq!(reverse.answer, "hello");
    }

    #[test]
    fn test_stats() {
        let mut manager = VocabularyManager::new(MockStorage::new());
        manager.add_pair("one".to_string(), "один".to_string()).unwrap();
        manager.add_pair("two".to_string(), "два".to_string()).unwrap();

        let stats = manager.get_stats().unwrap();
        assert_eq!(stats.total_pairs, 2);
        assert_eq!(stats.due_forward, 2);
        assert_eq!(stats.due_reverse, 2);
        assert_eq!(stats.due_total, 4);
    }

    #[test]
    fn test_delete_pair() {
        let mut manager = VocabularyManager::new(MockStorage::new());
        let id = manager.add_pair("test".to_string(), "тест".to_string()).unwrap();

        manager.delete_pair(id).unwrap();
        assert!(manager.get_pair(id).is_err());
    }

    #[test]
    fn test_independent_direction_reviews() {
        let mut manager = VocabularyManager::new(MockStorage::new());
        let id = manager.add_pair("sun".to_string(), "солнце".to_string()).unwrap();

        // Review forward 3 times, reverse 1 time
        manager.review(id, Direction::Forward, Rating::Good).unwrap();
        manager.review(id, Direction::Forward, Rating::Good).unwrap();
        manager.review(id, Direction::Forward, Rating::Again).unwrap();
        manager.review(id, Direction::Reverse, Rating::Easy).unwrap();

        let pair = manager.get_pair(id).unwrap();
        assert_eq!(pair.fsrs[0].card.reps, 3);
        assert_eq!(pair.fsrs[0].card.lapses, 1);
        assert_eq!(pair.fsrs[1].card.reps, 1);
        assert_eq!(pair.fsrs[1].card.lapses, 0);
    }
}
