use crate::models::{CardId, WordCard};
use crate::VocabularyError;

/// Trait for abstract storage of word cards
/// 
/// This trait abstracts the storage implementation, allowing
/// different backends (JSON file, SD card, flash, etc.) to be used.
pub trait Storage {
    /// Save a word card to storage
    fn save_card(&mut self, card: WordCard) -> Result<(), VocabularyError>;
    
    /// Load a word card by ID
    fn load_card(&self, id: CardId) -> Result<WordCard, VocabularyError>;
    
    /// Load all word cards
    fn load_all_cards(&self) -> Result<Vec<WordCard>, VocabularyError>;
    
    /// Delete a word card
    fn delete_card(&mut self, id: CardId) -> Result<(), VocabularyError>;
}

