use fsrs_core::{PairStorage, VocabularyError, WordPair, WordPairId, PairsFile};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

pub struct PostcardFileStorage {
    file_path: PathBuf,
    pairs: HashMap<WordPairId, WordPair>,
    next_id: WordPairId,
}

impl PostcardFileStorage {
    pub fn new<P: Into<PathBuf>>(file_path: P) -> Self {
        Self {
            file_path: file_path.into(),
            pairs: HashMap::new(),
            next_id: 1,
        }
    }

    pub fn load(&mut self) -> Result<(), VocabularyError> {
        if !self.file_path.exists() {
            return Ok(());
        }

        let bytes = fs::read(&self.file_path).map_err(VocabularyError::storage)?;
        let file: PairsFile = postcard::from_bytes(&bytes)
            .map_err(|e| VocabularyError::Serialization(e.to_string()))?;

        self.next_id = file.next_id;
        self.pairs = file.pairs.into_iter().map(|p| (p.id, p)).collect();
        Ok(())
    }

    fn save(&self) -> Result<(), VocabularyError> {
        let file = PairsFile {
            next_id: self.next_id,
            pairs: self.pairs.values().cloned().collect(),
        };
        let bytes = postcard::to_allocvec(&file)
            .map_err(|e| VocabularyError::Serialization(e.to_string()))?;
        fs::write(&self.file_path, bytes).map_err(VocabularyError::storage)?;
        Ok(())
    }

    pub fn next_id(&self) -> WordPairId {
        self.next_id
    }

    pub fn set_next_id(&mut self, id: WordPairId) {
        self.next_id = id;
    }
}

impl PairStorage for PostcardFileStorage {
    fn save_pair(&mut self, pair: WordPair) -> Result<(), VocabularyError> {
        if pair.id >= self.next_id {
            self.next_id = pair.id + 1;
        }
        self.pairs.insert(pair.id, pair);
        self.save()
    }

    fn load_pair(&self, id: WordPairId) -> Result<WordPair, VocabularyError> {
        self.pairs.get(&id).cloned().ok_or(VocabularyError::CardNotFound(id))
    }

    fn load_all_pairs(&self) -> Result<Vec<WordPair>, VocabularyError> {
        Ok(self.pairs.values().cloned().collect())
    }

    fn delete_pair(&mut self, id: WordPairId) -> Result<(), VocabularyError> {
        self.pairs.remove(&id).ok_or(VocabularyError::CardNotFound(id))?;
        self.save()
    }
}
