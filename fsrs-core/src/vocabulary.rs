use crate::models::{CardId, WordCard};
use crate::storage::Storage;
use crate::error::VocabularyError;
use crate::fsrs::{Rating, update_card_with_review, is_card_due};

/// Main entry point for vocabulary management
/// 
/// Manages word cards and FSRS scheduling, abstracting over storage implementation
pub struct VocabularyManager<S: Storage> {
    storage: S,
    next_card_id: CardId,
}

impl<S: Storage> VocabularyManager<S> {
    /// Create a new vocabulary manager with the given storage backend
    pub fn new(storage: S) -> Self {
        Self {
            storage,
            next_card_id: 1,
        }
    }

    /// Create a new vocabulary manager with a specific starting card ID
    pub fn with_next_id(storage: S, next_id: CardId) -> Self {
        Self {
            storage,
            next_card_id: next_id,
        }
    }

    /// Add a new word card
    pub fn add_card(&mut self, front: String, back: String) -> Result<CardId, VocabularyError> {
        let card_id = self.next_card_id;
        self.next_card_id += 1;
        
        let word_card = WordCard::new(card_id, front, back);
        self.storage.save_card(word_card)?;
        
        Ok(card_id)
    }

    /// Review a card with a given rating
    pub fn review_card(&mut self, id: CardId, rating: Rating) -> Result<(), VocabularyError> {
        let mut card = self.storage.load_card(id)?;
        
        // Update FSRS state
        update_card_with_review(&mut card.card, rating)?;
        
        // Record the review timestamp
        card.record_review();
        
        // Save updated card
        self.storage.save_card(card)?;
        
        Ok(())
    }

    /// Get all cards that are due for review
    pub fn get_due_cards(&self) -> Result<Vec<WordCard>, VocabularyError> {
        let all_cards = self.storage.load_all_cards()?;
        
        Ok(all_cards
            .into_iter()
            .filter(|card| is_card_due(&card.card))
            .collect())
    }

    /// Get all cards
    pub fn get_all_cards(&self) -> Result<Vec<WordCard>, VocabularyError> {
        self.storage.load_all_cards()
    }

    /// Get a specific card by ID
    pub fn get_card(&self, id: CardId) -> Result<WordCard, VocabularyError> {
        self.storage.load_card(id)
    }

    /// Delete a card
    pub fn delete_card(&mut self, id: CardId) -> Result<(), VocabularyError> {
        self.storage.delete_card(id)
    }

    /// Get statistics about the card collection
    pub fn get_stats(&self) -> Result<Stats, VocabularyError> {
        let all_cards = self.get_all_cards()?;
        
        let total_words = all_cards.len();
        let words_due = all_cards.iter()
            .filter(|card| is_card_due(&card.card))
            .count();
        
        Ok(Stats {
            total_words,
            words_due,
        })
    }

    /// Get the next card ID that will be assigned
    pub fn next_card_id(&self) -> CardId {
        self.next_card_id
    }

    /// Get mutable reference to storage (for persistence operations)
    pub fn storage_mut(&mut self) -> &mut S {
        &mut self.storage
    }
}

/// Statistics about the vocabulary collection
#[derive(Debug, Clone)]
pub struct Stats {
    pub total_words: usize,
    pub words_due: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::WordCard;
    use crate::VocabularyError;
    use std::collections::HashMap;

    /// Mock storage implementation for testing
    struct MockStorage {
        cards: HashMap<CardId, WordCard>,
    }

    impl MockStorage {
        fn new() -> Self {
            Self {
                cards: HashMap::new(),
            }
        }
    }

    impl Storage for MockStorage {
        fn save_card(&mut self, card: WordCard) -> Result<(), VocabularyError> {
            self.cards.insert(card.id, card);
            Ok(())
        }

        fn load_card(&self, id: CardId) -> Result<WordCard, VocabularyError> {
            self.cards.get(&id)
                .cloned()
                .ok_or(VocabularyError::CardNotFound(id))
        }

        fn load_all_cards(&self) -> Result<Vec<WordCard>, VocabularyError> {
            Ok(self.cards.values().cloned().collect())
        }

        fn delete_card(&mut self, id: CardId) -> Result<(), VocabularyError> {
            self.cards.remove(&id)
                .ok_or(VocabularyError::CardNotFound(id))
                .map(|_| ())
        }
    }

    #[test]
    /// Test scenario: Create a new VocabularyManager with fresh storage
    /// Expected: Manager is created with next_card_id = 1
    fn test_vocabulary_manager_new() {
        let storage = MockStorage::new();
        let manager = VocabularyManager::new(storage);
        
        assert_eq!(manager.next_card_id, 1);
    }

    #[test]
    /// Test scenario: Create a VocabularyManager with a custom starting ID
    /// Expected: Manager is created with the specified next_card_id
    fn test_vocabulary_manager_with_next_id() {
        let storage = MockStorage::new();
        let manager = VocabularyManager::with_next_id(storage, 100);
        
        assert_eq!(manager.next_card_id, 100);
    }

    #[test]
    /// Test scenario: Add a new word card to the collection
    /// Expected: Card is added and returns the assigned ID
    fn test_add_card() {
        let storage = MockStorage::new();
        let mut manager = VocabularyManager::new(storage);
        
        let card_id = manager.add_card("Hello".to_string(), "Hola".to_string()).unwrap();
        
        assert_eq!(card_id, 1);
        
        // Verify the card was saved
        let saved_card = manager.get_card(1).unwrap();
        assert_eq!(saved_card.front, "Hello");
        assert_eq!(saved_card.back, "Hola");
    }

    #[test]
    /// Test scenario: Add multiple word cards in sequence
    /// Expected: Each card gets a unique incrementing ID
    fn test_add_multiple_cards() {
        let storage = MockStorage::new();
        let mut manager = VocabularyManager::new(storage);
        
        let id1 = manager.add_card("Hello".to_string(), "Hola".to_string()).unwrap();
        let id2 = manager.add_card("Good".to_string(), "Bueno".to_string()).unwrap();
        let id3 = manager.add_card("Goodbye".to_string(), "Adiós".to_string()).unwrap();
        
        assert_eq!(id1, 1);
        assert_eq!(id2, 2);
        assert_eq!(id3, 3);
        
        // Verify all cards were saved
        let cards = manager.get_all_cards().unwrap();
        assert_eq!(cards.len(), 3);
    }

    #[test]
    /// Test scenario: Review a card with different ratings
    /// Expected: Card state is updated correctly for each rating
    fn test_review_card() {
        let storage = MockStorage::new();
        let mut manager = VocabularyManager::new(storage);
        
        let card_id = manager.add_card("Hello".to_string(), "Hola".to_string()).unwrap();
        
        // Review with Good rating
        manager.review_card(card_id, Rating::Good).unwrap();
        
        let card = manager.get_card(card_id).unwrap();
        assert!(card.last_review.is_some());
        assert_eq!(card.card.reps, 1);
        assert_eq!(card.card.lapses, 0);
    }

    #[test]
    /// Test scenario: Review a card that doesn't exist
    /// Expected: Returns CardNotFound error
    fn test_review_nonexistent_card() {
        let storage = MockStorage::new();
        let mut manager = VocabularyManager::new(storage);
        
        let result = manager.review_card(999, Rating::Good);
        assert!(result.is_err());
    }

    #[test]
    /// Test scenario: Get only due cards from the collection
    /// Expected: Returns only cards whose due date has passed
    fn test_get_due_cards() {
        let storage = MockStorage::new();
        let mut manager = VocabularyManager::new(storage);
        
        // Add a card
        let card_id = manager.add_card("Hello".to_string(), "Hola".to_string()).unwrap();
        
        // Get the card and verify it's due (new cards are due immediately)
        let due_cards = manager.get_due_cards().unwrap();
        
        // The card should be due since it's new
        assert!(due_cards.iter().any(|c| c.id == card_id));
    }

    #[test]
    /// Test scenario: Get all cards from the collection
    /// Expected: Returns all cards regardless of due status
    fn test_get_all_cards() {
        let storage = MockStorage::new();
        let mut manager = VocabularyManager::new(storage);
        
        manager.add_card("Hello".to_string(), "Hola".to_string()).unwrap();
        manager.add_card("Goodbye".to_string(), "Adiós".to_string()).unwrap();
        
        let cards = manager.get_all_cards().unwrap();
        assert_eq!(cards.len(), 2);
    }

    #[test]
    /// Test scenario: Delete a card from the collection
    /// Expected: Card is removed and subsequent access returns error
    fn test_delete_card() {
        let storage = MockStorage::new();
        let mut manager = VocabularyManager::new(storage);
        
        let card_id = manager.add_card("Hello".to_string(), "Hola".to_string()).unwrap();
        
        // Delete the card
        manager.delete_card(card_id).unwrap();
        
        // Verify card no longer exists
        assert!(manager.get_card(card_id).is_err());
    }

    #[test]
    /// Test scenario: Delete a card that doesn't exist
    /// Expected: Returns CardNotFound error
    fn test_delete_nonexistent_card() {
        let storage = MockStorage::new();
        let mut manager = VocabularyManager::new(storage);
        
        let result = manager.delete_card(999);
        assert!(result.is_err());
    }

    #[test]
    /// Test scenario: Get statistics about the collection
    /// Expected: Returns correct counts of total and due cards
    fn test_get_stats() {
        let storage = MockStorage::new();
        let mut manager = VocabularyManager::new(storage);
        
        // Add some cards
        manager.add_card("Hello".to_string(), "Hola".to_string()).unwrap();
        manager.add_card("Goodbye".to_string(), "Adiós".to_string()).unwrap();
        
        let stats = manager.get_stats().unwrap();
        assert_eq!(stats.total_words, 2);
        // New cards should be due
        assert!(stats.words_due >= 2);
    }

    #[test]
    /// Test scenario: Verify next_card_id increments after adding cards
    /// Expected: Each add_card call increments the next available ID
    fn test_next_card_id_increment() {
        let storage = MockStorage::new();
        let mut manager = VocabularyManager::new(storage);
        
        assert_eq!(manager.next_card_id(), 1);
        
        manager.add_card("One".to_string(), "Uno".to_string()).unwrap();
        assert_eq!(manager.next_card_id(), 2);
        
        manager.add_card("Two".to_string(), "Dos".to_string()).unwrap();
        assert_eq!(manager.next_card_id(), 3);
    }

    #[test]
    /// Test scenario: Review a card multiple times with different ratings
    /// Expected: Card accumulates reviews and lapses correctly
    fn test_multiple_reviews() {
        let storage = MockStorage::new();
        let mut manager = VocabularyManager::new(storage);
        
        let card_id = manager.add_card("Hello".to_string(), "Hola".to_string()).unwrap();
        
        // Good review
        manager.review_card(card_id, Rating::Good).unwrap();
        
        // Another Good review
        manager.review_card(card_id, Rating::Good).unwrap();
        
        // Again (forgot)
        manager.review_card(card_id, Rating::Again).unwrap();
        
        let card = manager.get_card(card_id).unwrap();
        assert_eq!(card.card.reps, 3);
        assert_eq!(card.card.lapses, 1);
    }

    #[test]
    /// Test scenario: Add a card and then review it to verify integration
    /// Expected: FSRS algorithm updates card state correctly
    fn test_review_updates_fsrs_state() {
        let storage = MockStorage::new();
        let mut manager = VocabularyManager::new(storage);
        
        let card_id = manager.add_card("Hello".to_string(), "Hola".to_string()).unwrap();
        let initial_card = manager.get_card(card_id).unwrap();
        let initial_stability = initial_card.card.stability;
        
        // Review with Good rating
        manager.review_card(card_id, Rating::Good).unwrap();
        
        let updated_card = manager.get_card(card_id).unwrap();
        
        // Card state should be updated
        assert!(updated_card.card.stability >= initial_stability);
        assert_eq!(updated_card.card.reps, 1);
    }

    #[test]
    /// Test scenario: Get storage mutable reference for persistence operations
    /// Expected: Can access storage mutably through storage_mut method
    fn test_storage_mut_access() {
        let storage = MockStorage::new();
        let mut manager = VocabularyManager::new(storage);
        
        // Verify we can get mutable storage access
        let _storage_ref = manager.storage_mut();
        assert!(true); // If we get here, access succeeded
    }
}

