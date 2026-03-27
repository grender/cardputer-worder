//! One-shot converter: words.json (old WordCard format) → pairs.bin (new PairsFile postcard format)
//! Run with: cargo run --bin convert_words --target aarch64-apple-darwin

use std::fs;

fn main() {
    let input = fs::read_to_string("words.json").expect("Failed to read words.json");

    // Parse old format: Vec of objects with {id, front, back, examples, card, created_at, last_review}
    let old_cards: Vec<serde_json::Value> = serde_json::from_str(&input).expect("Invalid JSON");

    let mut pairs: Vec<fsrs_core::WordPair> = Vec::new();
    let mut max_id: u64 = 0;

    for card in &old_cards {
        let id = card["id"].as_u64().unwrap_or(0);
        if id > max_id {
            max_id = id;
        }

        let en = card["front"].as_str().unwrap_or("").to_string();
        let ru = card["back"].as_str().unwrap_or("").to_string();
        let examples: Vec<String> = card["examples"]
            .as_array()
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                    .collect()
            })
            .unwrap_or_default();

        let created_at = card["created_at"]
            .as_str()
            .unwrap_or("1970-01-01T00:00:00Z")
            .to_string();

        // Parse the FSRS Card from the old format
        let fsrs_card: rs_fsrs::Card =
            serde_json::from_value(card["card"].clone()).expect("Failed to parse FSRS card");

        let last_review = card["last_review"].as_str().map(|s| s.to_string());

        // Forward direction gets the existing FSRS state
        let forward_state = fsrs_core::DirectionState {
            card: fsrs_card,
            last_review,
        };

        // Reverse direction starts fresh
        let reverse_state = fsrs_core::DirectionState::new();

        let pair = fsrs_core::WordPair {
            id,
            en,
            ru,
            examples,
            fsrs: [forward_state, reverse_state],
            created_at,
        };

        pairs.push(pair);
    }

    let pairs_file = fsrs_core::PairsFile {
        next_id: max_id + 1,
        pairs,
    };

    // Save as postcard binary
    let bytes = postcard::to_allocvec(&pairs_file).expect("postcard serialize failed");
    fs::write("pairs.bin", &bytes).expect("Failed to write pairs.bin");

    // Also save as JSON (for SD card / debugging)
    let json = serde_json::to_string_pretty(&pairs_file).expect("JSON serialize failed");
    fs::write("pairs.json", &json).expect("Failed to write pairs.json");

    println!(
        "Converted {} cards → {} pairs",
        old_cards.len(),
        pairs_file.pairs.len()
    );
    println!("  next_id: {}", pairs_file.next_id);
    println!("  pairs.bin: {} bytes (postcard)", bytes.len());
    println!("  pairs.json: {} bytes (JSON, for SD card)", json.len());
}
