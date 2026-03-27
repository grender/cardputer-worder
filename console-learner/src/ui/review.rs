use fsrs_core::{VocabularyManager, PairStorage, Rating, ReviewItem, Direction};

pub fn run_review_session<S: PairStorage>(
    manager: &mut VocabularyManager<S>,
) -> Result<(), fsrs_core::VocabularyError> {
    let due_items = manager.get_due_reviews()?;

    if due_items.is_empty() {
        println!("\nNo cards due for review!");
        crate::ui::menu::wait_for_enter();
        return Ok(());
    }

    println!("\n=== Review Session ({} cards due) ===\n", due_items.len());

    for (index, item) in due_items.iter().enumerate() {
        let dir_label = match item.direction {
            Direction::Forward => "EN > RU",
            Direction::Reverse => "RU > EN",
        };
        println!("Card {} of {} [{}]", index + 1, due_items.len(), dir_label);
        display_prompt(item);

        crate::ui::menu::wait_for_enter();

        display_answer(item);
        let rating = get_rating();

        match rating {
            Ok(rating) => {
                manager.review(item.pair_id, item.direction, rating)?;
                let pair = manager.get_pair(item.pair_id)?;
                let state = pair.direction_state(item.direction);
                println!("\nReviewed! Next in: {}", format_next_review(&state.card));
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

fn display_prompt(item: &ReviewItem) {
    println!("Word: {}\n", item.prompt);
    if !item.examples.is_empty() {
        println!("Examples:");
        for example in &item.examples {
            println!("  - {}", example);
        }
        println!();
    }
    println!("Press ENTER to reveal answer...");
}

fn display_answer(item: &ReviewItem) {
    println!("\nAnswer: {}", item.answer);
    println!("\nHow did you do?");
    println!("  1. Again (Forgot)");
    println!("  2. Hard");
    println!("  3. Good");
    println!("  4. Easy");
}

fn get_rating() -> Result<Rating, String> {
    print!("\nEnter your rating [1-4]: ");
    let mut input = String::new();
    std::io::stdin().read_line(&mut input).map_err(|e| e.to_string())?;
    let val = input.trim().parse::<u8>().map_err(|e| e.to_string())?;
    Rating::from_u8(val).map_err(|_| "Invalid rating".to_string())
}

fn format_next_review(card: &rs_fsrs::Card) -> String {
    use chrono::Utc;
    let now = Utc::now();
    let duration = card.due.signed_duration_since(now);
    let days = duration.num_days();
    if days <= 0 {
        "today".to_string()
    } else {
        format!("{} days", days)
    }
}
