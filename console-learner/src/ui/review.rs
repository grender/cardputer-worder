use fsrs_core::{VocabularyManager, Rating, Storage, WordCard};

/// Run a review session
pub fn run_review_session<S: Storage>(
    manager: &mut VocabularyManager<S>
) -> Result<(), fsrs_core::VocabularyError> {
    let due_cards = manager.get_due_cards()?;
    
    if due_cards.is_empty() {
        println!("\n✅ No cards due for review!");
        crate::ui::menu::wait_for_enter();
        return Ok(());
    }

    println!("\n=== Review Session ===\n");
    
    for (index, card) in due_cards.iter().enumerate() {
        println!("Card {} of {}", index + 1, due_cards.len());
        display_card_front(card);
        
        crate::ui::menu::wait_for_enter();
        
        display_card_back(card);
        let rating = get_rating();
        
        match rating {
            Ok(rating) => {
                manager.review_card(card.id, rating)?;
                println!("\n✅ Card reviewed successfully");
                println!("Next review: {}", format_next_review(&card.card));
            }
            Err(e) => {
                println!("Error: {}", e);
            }
        }
        
        println!("\nPress ENTER for next card...");
        crate::ui::menu::wait_for_enter();
    }

    Ok(())
}

/// Display the front of a card (word to translate)
fn display_card_front(card: &WordCard) {
    println!("Word: {}\n", card.front);
    if !card.examples.is_empty() {
        println!("Examples:");
        for example in &card.examples {
            println!("  - {}", example);
        }
        println!();
    }
    println!("Press ENTER to reveal translation...");
}

/// Display the back of a card (translation)
fn display_card_back(card: &WordCard) {
    println!("\nTranslation: {}", card.back);
    println!("\nHow did you do?");
    println!("  1. Again (Forgot)         - Show more frequently");
    println!("  2. Hard                   - Difficult but remembered");
    println!("  3. Good                   - Correct answer");
    println!("  4. Easy                   - Knew it immediately");
}

/// Get rating from user input
fn get_rating() -> Result<Rating, String> {
    print!("\nEnter your rating [1-4]: ");
    let mut input = String::new();
    std::io::stdin().read_line(&mut input).map_err(|e| e.to_string())?;
    
    let val = input.trim().parse::<u8>().map_err(|e| e.to_string())?;
    Rating::from_u8(val).map_err(|_| "Invalid rating".to_string())
}

/// Format the next review date
fn format_next_review(card: &rs_fsrs::Card) -> String {
    // Convert DateTime to a readable format
    use chrono::Utc;
    let now = Utc::now();
    let duration = card.due.signed_duration_since(now);
    let days = duration.num_days();
    format!("{} days", days.max(0))
}

