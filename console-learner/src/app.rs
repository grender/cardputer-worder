use fsrs_core::{VocabularyManager, Direction, is_card_due};
use crate::storage::PostcardFileStorage;
use crate::ui::menu::{display_menu, read_choice, wait_for_enter, read_line};
use crate::ui::review::run_review_session;

pub struct App {
    manager: VocabularyManager<PostcardFileStorage>,
}

impl App {
    pub fn new() -> Result<Self, fsrs_core::VocabularyError> {
        let mut storage = PostcardFileStorage::new("pairs.bin");
        storage.load()?;
        let next_id = storage.next_id();
        let manager = VocabularyManager::with_next_id(storage, next_id);
        Ok(Self { manager })
    }

    pub fn run(&mut self) -> Result<(), fsrs_core::VocabularyError> {
        loop {
            display_menu();
            let choice = read_choice();

            match choice.as_str() {
                "1" => self.add_word_flow()?,
                "2" => self.review_session_flow()?,
                "3" => self.list_words_flow()?,
                "4" => self.show_stats_flow()?,
                "5" => break,
                _ => println!("Invalid choice, please select 1-5"),
            }
        }
        Ok(())
    }

    fn add_word_flow(&mut self) -> Result<(), fsrs_core::VocabularyError> {
        println!("\n=== Add New Word Pair ===");

        let en = read_line("Enter English word: ");
        let ru = read_line("Enter Russian translation: ");

        if !en.is_empty() && !ru.is_empty() {
            let pair_id = self.manager.add_pair(en, ru)?;
            println!("\nWord pair added! (ID: {})", pair_id);
        } else {
            println!("Not added: both fields are required");
        }

        wait_for_enter();
        Ok(())
    }

    fn review_session_flow(&mut self) -> Result<(), fsrs_core::VocabularyError> {
        run_review_session(&mut self.manager)?;
        Ok(())
    }

    fn list_words_flow(&mut self) -> Result<(), fsrs_core::VocabularyError> {
        let pairs = self.manager.get_all_pairs()?;

        println!("\n=== All Word Pairs ({} total) ===\n", pairs.len());

        for pair in &pairs {
            let fwd_due = if is_card_due(&pair.direction_state(Direction::Forward).card) {
                "due"
            } else {
                "ok"
            };
            let rev_due = if is_card_due(&pair.direction_state(Direction::Reverse).card) {
                "due"
            } else {
                "ok"
            };
            println!(
                "{}. {} - {} [EN>RU: {}, RU>EN: {}]",
                pair.id, pair.en, pair.ru, fwd_due, rev_due
            );
        }

        println!("\nPress ENTER to return to menu...");
        wait_for_enter();
        Ok(())
    }

    fn show_stats_flow(&mut self) -> Result<(), fsrs_core::VocabularyError> {
        let stats = self.manager.get_stats()?;

        println!("\n=== Learning Statistics ===");
        println!("\nTotal Pairs: {}", stats.total_pairs);
        println!("Due EN>RU:   {}", stats.due_forward);
        println!("Due RU>EN:   {}", stats.due_reverse);
        println!("Due Total:   {}", stats.due_total);

        println!("\nPress ENTER to return to menu...");
        wait_for_enter();
        Ok(())
    }
}
