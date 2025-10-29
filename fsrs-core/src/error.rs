use std::fmt::Display;

#[derive(Debug, PartialEq, Clone)]
pub enum VocabularyError {
    Storage(String), // Changed to store error message instead of Box
    CardNotFound(u64),
    Serialization(String),
    InvalidRating(u8),
}

impl std::error::Error for VocabularyError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None // No nested error source
    }
}

impl Display for VocabularyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VocabularyError::Storage(msg) => write!(f, "Storage error: {}", msg),
            VocabularyError::CardNotFound(id) => write!(f, "Card not found: {}", id),
            VocabularyError::Serialization(msg) => write!(f, "Serialization error: {}", msg),
            VocabularyError::InvalidRating(r) => write!(f, "Invalid rating: {}", r),
        }
    }
}

impl VocabularyError {
    pub fn storage<E>(err: E) -> Self
    where
        E: std::error::Error + Send + Sync + 'static,
    {
        Self::Storage(format!("{}", err))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::error::Error;

    #[test]
    /// Test scenario: Verify error can be created from std::io::Error
    /// Expected: VocabularyError::storage creates Storage variant successfully
    fn test_storage_error_creation() {
        let io_error = std::io::Error::new(std::io::ErrorKind::NotFound, "File not found");
        let error = VocabularyError::storage(io_error);
        assert!(matches!(error, VocabularyError::Storage(_)));
    }

    #[test]
    /// Test scenario: Verify error formatting for different variants
    /// Expected: Each variant formats correctly with appropriate message
    fn test_error_display() {
        // Test CardNotFound
        let error = VocabularyError::CardNotFound(123);
        assert_eq!(format!("{}", error), "Card not found: 123");

        // Test Serialization
        let error = VocabularyError::Serialization("Invalid JSON".to_string());
        assert_eq!(format!("{}", error), "Serialization error: Invalid JSON");

        // Test InvalidRating
        let error = VocabularyError::InvalidRating(99);
        assert_eq!(format!("{}", error), "Invalid rating: 99");
    }

    #[test]
    /// Test scenario: Verify error implements Error trait properly
    /// Expected: source() method returns None
    fn test_error_source() {
        let error = VocabularyError::CardNotFound(1);
        assert!(error.source().is_none());
    }

    #[test]
    /// Test scenario: Verify PartialEq for comparing error instances
    /// Expected: Error variants can be compared for equality
    fn test_error_comparison() {
        let error1 = VocabularyError::CardNotFound(123);
        let error2 = VocabularyError::CardNotFound(123);
        let error3 = VocabularyError::CardNotFound(456);

        assert_eq!(error1, error2);
        assert_ne!(error1, error3);
    }
}

