use fsrs_core::{Storage, VocabularyError, WordCard, CardId};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

/// JSON file-based storage implementation
/// 
/// Stores all word cards in a single JSON file
pub struct JsonFileStorage {
    file_path: PathBuf,
    cards: HashMap<CardId, WordCard>,
}

impl JsonFileStorage {
    /// Create a new JSON file storage
    pub fn new<P: Into<PathBuf>>(file_path: P) -> Self {
        Self {
            file_path: file_path.into(),
            cards: HashMap::new(),
        }
    }

    /// Load cards from the JSON file
    pub fn load(&mut self) -> Result<(), VocabularyError> {
        if !self.file_path.exists() {
            // File doesn't exist yet, start with empty collection
            return Ok(());
        }

        let contents = fs::read_to_string(&self.file_path)
            .map_err(VocabularyError::storage)?;
        
        let cards_vec: Vec<WordCard> = serde_json::from_str(&contents)
            .map_err(|e| VocabularyError::Serialization(e.to_string()))?;
        
        self.cards = cards_vec.into_iter().map(|card| (card.id, card)).collect();
        
        Ok(())
    }

    /// Save cards to the JSON file
    pub fn save(&self) -> Result<(), VocabularyError> {
        let cards_vec: Vec<&WordCard> = self.cards.values().collect();
        let json = serde_json::to_string_pretty(&cards_vec)
            .map_err(|e| VocabularyError::Serialization(e.to_string()))?;
        
        fs::write(&self.file_path, json)
            .map_err(VocabularyError::storage)?;
        
        Ok(())
    }
}

impl Storage for JsonFileStorage {
    fn save_card(&mut self, card: WordCard) -> Result<(), VocabularyError> {
        self.cards.insert(card.id, card);
        self.save()
    }

    fn load_card(&self, id: CardId) -> Result<WordCard, VocabularyError> {
        self.cards.get(&id)
            .ok_or_else(|| VocabularyError::CardNotFound(id))
            .map(|c| c.clone())
    }

    fn load_all_cards(&self) -> Result<Vec<WordCard>, VocabularyError> {
        Ok(self.cards.values().cloned().collect())
    }

    fn delete_card(&mut self, id: CardId) -> Result<(), VocabularyError> {
        self.cards.remove(&id).ok_or_else(|| VocabularyError::CardNotFound(id))?;
        self.save()
    }
}

