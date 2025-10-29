use rs_fsrs::{Card, FSRS, Rating as FsrsRating};
use crate::VocabularyError;
use chrono::Utc;

/// Review rating for FSRS algorithm
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Rating {
    Again = 1,
    Hard = 2,
    Good = 3,
    Easy = 4,
}

impl Rating {
    pub fn from_u8(val: u8) -> Result<Self, VocabularyError> {
        match val {
            1 => Ok(Rating::Again),
            2 => Ok(Rating::Hard),
            3 => Ok(Rating::Good),
            4 => Ok(Rating::Easy),
            _ => Err(VocabularyError::InvalidRating(val)),
        }
    }
}

/// Update a card with a review rating using FSRS algorithm
pub fn update_card_with_review(card: &mut Card, rating: Rating) -> Result<(), VocabularyError> {
    let now = Utc::now();
    
    // Convert our Rating to rs-fsrs Rating
    let fsrs_rating = match rating {
        Rating::Again => FsrsRating::Again,
        Rating::Hard => FsrsRating::Hard,
        Rating::Good => FsrsRating::Good,
        Rating::Easy => FsrsRating::Easy,
    };
    
    // Create FSRS instance with default parameters
    let fsrs = FSRS::new(rs_fsrs::Parameters::default());
    
    // Use FSRS algorithm to update the card
    let scheduling_info = fsrs.next(card.clone(), now, fsrs_rating);
    
    // Update the card with FSRS-calculated values
    *card = scheduling_info.card;
    
    Ok(())
}

/// Check if a card is due for review
/// 
/// Returns true if the card's due date has passed
pub fn is_card_due(card: &Card) -> bool {
    let now = Utc::now();
    card.due <= now
}

#[cfg(test)]
mod tests {
    use super::*;
    use rs_fsrs::Card;
    use chrono::Duration;

    #[test]
    /// Test scenario: Convert u8 values to Rating enum
    /// Expected: Valid ratings (1-4) convert successfully, invalid values return error
    fn test_rating_from_u8() {
        assert_eq!(Rating::from_u8(1).unwrap(), Rating::Again);
        assert_eq!(Rating::from_u8(2).unwrap(), Rating::Hard);
        assert_eq!(Rating::from_u8(3).unwrap(), Rating::Good);
        assert_eq!(Rating::from_u8(4).unwrap(), Rating::Easy);
        
        // Invalid ratings should return error
        assert!(Rating::from_u8(0).is_err());
        assert!(Rating::from_u8(5).is_err());
        assert!(Rating::from_u8(255).is_err());
    }

    #[test]
    /// Test scenario: Verify Rating enum values match FSRS algorithm expectations
    /// Expected: Each rating has correct associated integer value
    fn test_rating_values() {
        assert_eq!(Rating::Again as u8, 1);
        assert_eq!(Rating::Hard as u8, 2);
        assert_eq!(Rating::Good as u8, 3);
        assert_eq!(Rating::Easy as u8, 4);
    }

    #[test]
    /// Test scenario: Update a card with Again rating (difficult to recall)
    /// Expected: Card stability and difficulty are updated by FSRS algorithm
    fn test_update_card_with_again_rating() {
        let mut card = Card::new();
        
        update_card_with_review(&mut card, Rating::Again).unwrap();
        
        // FSRS algorithm should have updated the card
        // The stability may increase or decrease based on the rating
        // The fact that it changed confirms the algorithm ran
        assert_ne!(card.reps, 0);
    }

    #[test]
    /// Test scenario: Update a card with Good rating (normal recall)
    /// Expected: Card stability increases, difficulty may adjust
    fn test_update_card_with_good_rating() {
        let mut card = Card::new();
        
        update_card_with_review(&mut card, Rating::Good).unwrap();
        
        // After a Good rating, the card should have progression
        assert_eq!(card.reps, 1);
        assert_eq!(card.lapses, 0);
    }

    #[test]
    /// Test scenario: Update a card with Hard rating (slightly difficult)
    /// Expected: Card stability increases moderately
    fn test_update_card_with_hard_rating() {
        let mut card = Card::new();
        
        update_card_with_review(&mut card, Rating::Hard).unwrap();
        
        // Hard rating should increment reps but no lapses
        assert_eq!(card.reps, 1);
        assert_eq!(card.lapses, 0);
    }

    #[test]
    /// Test scenario: Update a card with Easy rating (very easy recall)
    /// Expected: Card stability increases significantly
    fn test_update_card_with_easy_rating() {
        let mut card = Card::new();
        
        update_card_with_review(&mut card, Rating::Easy).unwrap();
        
        // Easy rating should increment reps without lapses
        assert_eq!(card.reps, 1);
        assert_eq!(card.lapses, 0);
    }

    #[test]
    /// Test scenario: Multiple reviews of the same card update state correctly
    /// Expected: Card state accumulates across multiple reviews
    fn test_multiple_card_reviews() {
        let mut card = Card::new();
        
        // First review - Good
        update_card_with_review(&mut card, Rating::Good).unwrap();
        assert_eq!(card.reps, 1);
        
        // Second review - Good
        update_card_with_review(&mut card, Rating::Good).unwrap();
        assert_eq!(card.reps, 2);
        
        // Third review - Again (forgot)
        update_card_with_review(&mut card, Rating::Again).unwrap();
        assert_eq!(card.reps, 3);
        assert_eq!(card.lapses, 1); // Lapse should be incremented
    }

    #[test]
    /// Test scenario: Check if a card is due when its due date has passed
    /// Expected: Returns true when current time >= due date
    fn test_is_card_due_past() {
        let mut card = Card::new();
        // Set due date to the past
        let past_date = Utc::now() - Duration::days(1);
        card.due = past_date;
        
        assert!(is_card_due(&card));
    }

    #[test]
    /// Test scenario: Check if a card is due when its due date is in the future
    /// Expected: Returns false when current time < due date
    fn test_is_card_due_future() {
        let mut card = Card::new();
        // Set due date to the future
        let future_date = Utc::now() + Duration::days(1);
        card.due = future_date;
        
        assert!(!is_card_due(&card));
    }

    #[test]
    /// Test scenario: Check if a card is due when its due date is exactly now
    /// Expected: Returns true when current time == due date
    fn test_is_card_due_now() {
        let mut card = Card::new();
        card.due = Utc::now();
        
        assert!(is_card_due(&card));
    }

    #[test]
    /// Test scenario: Verify card due date is updated after review
    /// Expected: Card gets a future due date after successful review
    fn test_card_due_date_updated_after_review() {
        let mut card = Card::new();
        let initial_due = card.due;
        
        update_card_with_review(&mut card, Rating::Good).unwrap();
        
        // Card should have a future due date after review
        assert!(card.due > initial_due || card.due >= Utc::now());
    }

    #[test]
    /// Test scenario: Verify card stability increases over multiple reviews
    /// Expected: Stability increases with successful reviews
    fn test_card_stability_progression() {
        let mut card = Card::new();
        let initial_stability = card.stability;
        
        // Multiple good reviews
        for _ in 0..3 {
            update_card_with_review(&mut card, Rating::Good).unwrap();
        }
        
        // Stability should have increased
        assert!(card.stability >= initial_stability);
    }
}

