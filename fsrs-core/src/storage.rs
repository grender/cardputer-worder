use crate::models::{WordPairId, WordPair};
use crate::VocabularyError;

/// Trait for abstract storage of word pairs
pub trait PairStorage {
    fn save_pair(&mut self, pair: WordPair) -> Result<(), VocabularyError>;
    fn load_pair(&self, id: WordPairId) -> Result<WordPair, VocabularyError>;
    fn load_all_pairs(&self) -> Result<Vec<WordPair>, VocabularyError>;
    fn delete_pair(&mut self, id: WordPairId) -> Result<(), VocabularyError>;
}
