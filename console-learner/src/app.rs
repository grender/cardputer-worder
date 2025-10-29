use fsrs_core::{VocabularyManager, is_card_due};
use crate::storage::JsonFileStorage;
use crate::ui::menu::{display_menu, read_choice, wait_for_enter, read_line};
use crate::ui::review::run_review_session;

/// Main application structure
pub struct App {
    manager: VocabularyManager<JsonFileStorage>,
}

impl App {
    /// Create a new application instance
    pub fn new() -> Result<Self, fsrs_core::VocabularyError> {
        let mut storage = JsonFileStorage::new("words.json");
        storage.load()?;
        
        let manager = VocabularyManager::new(storage);
        
        Ok(Self { manager })
    }
    
    /// Save data to storage
    pub fn save_data(&mut self) -> Result<(), fsrs_core::VocabularyError> {
        // The JsonFileStorage implements Storage trait
        // and automatically saves on save_card/delete_card
        Ok(())
    }

    /// Run the main application loop
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

    /// Add a new word flow
    fn add_word_flow(&mut self) -> Result<(), fsrs_core::VocabularyError> {
        println!("\n=== Add New Word ===");
        
        let front = read_line("Enter English word: ");
        let back = read_line("Enter translation/definition: ");
        
        if !front.is_empty() && !back.is_empty() {
            let card_id = self.manager.add_card(front.clone(), back)?;
            println!("\n✅ Word added successfully!");
            println!("Card ID: {}", card_id);
        } else {
            println!("❌ Word not added: Front and back cannot be empty");
        }

        wait_for_enter();
        Ok(())
    }

    /// Review session flow
    fn review_session_flow(&mut self) -> Result<(), fsrs_core::VocabularyError> {
        run_review_session(&mut self.manager)?;
        Ok(())
    }

    /// List all words flow
    fn list_words_flow(&mut self) -> Result<(), fsrs_core::VocabularyError> {
        let cards = self.manager.get_all_cards()?;
        
        println!("\n=== All Words ({} total) ===\n", cards.len());
        
        for card in &cards {
            let due_status = if is_card_due(&card.card) {
                "Due now"
            } else {
                "Not due"
            };
            println!("{}. {} - {} - {}", card.id, card.front, card.back, due_status);
        }

        println!("\nPress ENTER to return to menu...");
        wait_for_enter();
        Ok(())
    }

    /// Show statistics flow
    fn show_stats_flow(&mut self) -> Result<(), fsrs_core::VocabularyError> {
        let stats = self.manager.get_stats()?;
        
        println!("\n=== Learning Statistics ===");
        println!("\nTotal Words: {}", stats.total_words);
        println!("Words Due: {}", stats.words_due);
        
        println!("\nPress ENTER to return to menu...");
        wait_for_enter();
        Ok(())
    }
}

