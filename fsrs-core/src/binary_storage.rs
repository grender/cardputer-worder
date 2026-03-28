//! Fixed-size binary storage for FSRS scheduling records.
//!
//! Layout:
//! - `FSRS.BIN`: header (16 bytes) + N × FsrsRecord (160 bytes each)
//! - `WORDS.BIN`: length-prefixed postcard blobs of WordText

use chrono::{DateTime, Utc};
use rs_fsrs::{Card, State};

pub const FSRS_MAGIC: u32 = 0x46535253; // "FSRS"
pub const FSRS_VERSION: u16 = 1;
pub const FSRS_HEADER_SIZE: usize = 16;
pub const FSRS_RECORD_SIZE: usize = 160;

// ---- Header ----

pub struct FsrsHeader {
    pub magic: u32,
    pub version: u16,
    pub record_size: u16,
    pub next_id: u64,
}

impl FsrsHeader {
    pub fn new(next_id: u64) -> Self {
        Self {
            magic: FSRS_MAGIC,
            version: FSRS_VERSION,
            record_size: FSRS_RECORD_SIZE as u16,
            next_id,
        }
    }

    pub fn to_bytes(&self) -> [u8; FSRS_HEADER_SIZE] {
        let mut buf = [0u8; FSRS_HEADER_SIZE];
        buf[0..4].copy_from_slice(&self.magic.to_le_bytes());
        buf[4..6].copy_from_slice(&self.version.to_le_bytes());
        buf[6..8].copy_from_slice(&self.record_size.to_le_bytes());
        buf[8..16].copy_from_slice(&self.next_id.to_le_bytes());
        buf
    }

    pub fn from_bytes(buf: &[u8; FSRS_HEADER_SIZE]) -> Option<Self> {
        let magic = u32::from_le_bytes(buf[0..4].try_into().ok()?);
        if magic != FSRS_MAGIC {
            return None;
        }
        Some(Self {
            magic,
            version: u16::from_le_bytes(buf[4..6].try_into().ok()?),
            record_size: u16::from_le_bytes(buf[6..8].try_into().ok()?),
            next_id: u64::from_le_bytes(buf[8..16].try_into().ok()?),
        })
    }
}

// ---- Binary direction state (68 bytes) ----

#[derive(Debug, Clone)]
pub struct BinaryDirState {
    pub due: i64,
    pub last_review_card: i64,
    pub stability: f64,
    pub difficulty: f64,
    pub elapsed_days: i64,
    pub scheduled_days: i64,
    pub reps: i32,
    pub lapses: i32,
    pub state: u8,
    pub has_last_review: bool,
    pub last_review: i64,
}

const DIR_STATE_SIZE: usize = 68;

impl BinaryDirState {
    pub fn write_to(&self, buf: &mut [u8]) {
        buf[0..8].copy_from_slice(&self.due.to_le_bytes());
        buf[8..16].copy_from_slice(&self.last_review_card.to_le_bytes());
        buf[16..24].copy_from_slice(&self.stability.to_le_bytes());
        buf[24..32].copy_from_slice(&self.difficulty.to_le_bytes());
        buf[32..40].copy_from_slice(&self.elapsed_days.to_le_bytes());
        buf[40..48].copy_from_slice(&self.scheduled_days.to_le_bytes());
        buf[48..52].copy_from_slice(&self.reps.to_le_bytes());
        buf[52..56].copy_from_slice(&self.lapses.to_le_bytes());
        buf[56] = self.state;
        buf[57] = if self.has_last_review { 1 } else { 0 };
        buf[58..66].copy_from_slice(&self.last_review.to_le_bytes());
        buf[66..68].copy_from_slice(&[0u8; 2]); // padding
    }

    pub fn read_from(buf: &[u8]) -> Self {
        Self {
            due: i64::from_le_bytes(buf[0..8].try_into().unwrap()),
            last_review_card: i64::from_le_bytes(buf[8..16].try_into().unwrap()),
            stability: f64::from_le_bytes(buf[16..24].try_into().unwrap()),
            difficulty: f64::from_le_bytes(buf[24..32].try_into().unwrap()),
            elapsed_days: i64::from_le_bytes(buf[32..40].try_into().unwrap()),
            scheduled_days: i64::from_le_bytes(buf[40..48].try_into().unwrap()),
            reps: i32::from_le_bytes(buf[48..52].try_into().unwrap()),
            lapses: i32::from_le_bytes(buf[52..56].try_into().unwrap()),
            state: buf[56],
            has_last_review: buf[57] != 0,
            last_review: i64::from_le_bytes(buf[58..66].try_into().unwrap()),
        }
    }

    pub fn from_card_and_review(card: &Card, last_review: &Option<String>) -> Self {
        Self {
            due: card.due.timestamp(),
            last_review_card: card.last_review.timestamp(),
            stability: card.stability,
            difficulty: card.difficulty,
            elapsed_days: card.elapsed_days,
            scheduled_days: card.scheduled_days,
            reps: card.reps,
            lapses: card.lapses,
            state: match card.state {
                State::New => 0,
                State::Learning => 1,
                State::Review => 2,
                State::Relearning => 3,
            },
            has_last_review: last_review.is_some(),
            last_review: last_review
                .as_ref()
                .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
                .map(|dt| dt.timestamp())
                .unwrap_or(0),
        }
    }

    pub fn to_card(&self) -> Card {
        Card {
            due: ts_to_dt(self.due),
            last_review: ts_to_dt(self.last_review_card),
            stability: self.stability,
            difficulty: self.difficulty,
            elapsed_days: self.elapsed_days,
            scheduled_days: self.scheduled_days,
            reps: self.reps,
            lapses: self.lapses,
            state: match self.state {
                0 => State::New,
                1 => State::Learning,
                2 => State::Review,
                3 => State::Relearning,
                _ => State::New,
            },
        }
    }

    pub fn to_last_review_string(&self) -> Option<String> {
        if self.has_last_review {
            Some(ts_to_dt(self.last_review).to_rfc3339())
        } else {
            None
        }
    }

    pub fn is_due(&self) -> bool {
        let now = Utc::now().timestamp();
        self.due <= now
    }
}

// ---- FSRS Record (160 bytes) ----

#[derive(Debug, Clone)]
pub struct FsrsRecord {
    pub pair_id: u64,
    pub word_offset: u32,
    pub word_length: u32,
    pub forward: BinaryDirState,
    pub reverse: BinaryDirState,
}

impl FsrsRecord {
    pub fn to_bytes(&self) -> [u8; FSRS_RECORD_SIZE] {
        let mut buf = [0u8; FSRS_RECORD_SIZE];
        buf[0..8].copy_from_slice(&self.pair_id.to_le_bytes());
        buf[8..12].copy_from_slice(&self.word_offset.to_le_bytes());
        buf[12..16].copy_from_slice(&self.word_length.to_le_bytes());
        self.forward.write_to(&mut buf[16..16 + DIR_STATE_SIZE]);
        self.reverse.write_to(&mut buf[16 + DIR_STATE_SIZE..16 + 2 * DIR_STATE_SIZE]);
        // buf[152..160] = padding (already zeroed)
        buf
    }

    pub fn from_bytes(buf: &[u8; FSRS_RECORD_SIZE]) -> Self {
        Self {
            pair_id: u64::from_le_bytes(buf[0..8].try_into().unwrap()),
            word_offset: u32::from_le_bytes(buf[8..12].try_into().unwrap()),
            word_length: u32::from_le_bytes(buf[12..16].try_into().unwrap()),
            forward: BinaryDirState::read_from(&buf[16..16 + DIR_STATE_SIZE]),
            reverse: BinaryDirState::read_from(&buf[16 + DIR_STATE_SIZE..16 + 2 * DIR_STATE_SIZE]),
        }
    }

    /// File offset for record at given slot index
    pub fn file_offset(slot: usize) -> u32 {
        (FSRS_HEADER_SIZE + slot * FSRS_RECORD_SIZE) as u32
    }
}

// ---- WordText (for WORDS.BIN) ----

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct WordText {
    pub en: String,
    pub ru: String,
    pub examples: Vec<String>,
    pub created_at: String,
}

// ---- DueItem (compact in-memory representation) ----

#[derive(Debug, Clone)]
pub struct DueItem {
    pub pair_id: u64,
    pub direction: crate::Direction,
    pub slot: usize,
    pub word_offset: u32,
    pub word_length: u32,
}

// ---- Helpers ----

fn ts_to_dt(ts: i64) -> DateTime<Utc> {
    DateTime::from_timestamp(ts, 0).unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;
    use rs_fsrs::Card;

    #[test]
    fn test_header_roundtrip() {
        let header = FsrsHeader::new(42);
        let bytes = header.to_bytes();
        let parsed = FsrsHeader::from_bytes(&bytes).unwrap();
        assert_eq!(parsed.magic, FSRS_MAGIC);
        assert_eq!(parsed.version, FSRS_VERSION);
        assert_eq!(parsed.next_id, 42);
    }

    #[test]
    fn test_record_roundtrip() {
        let card = Card::new();
        let fwd = BinaryDirState::from_card_and_review(&card, &None);
        let rev = BinaryDirState::from_card_and_review(&card, &Some("2026-01-15T10:30:00+00:00".to_string()));

        let record = FsrsRecord {
            pair_id: 123,
            word_offset: 4096,
            word_length: 200,
            forward: fwd,
            reverse: rev,
        };

        let bytes = record.to_bytes();
        assert_eq!(bytes.len(), FSRS_RECORD_SIZE);

        let parsed = FsrsRecord::from_bytes(&bytes);
        assert_eq!(parsed.pair_id, 123);
        assert_eq!(parsed.word_offset, 4096);
        assert_eq!(parsed.word_length, 200);

        // Check forward card roundtrip
        let fwd_card = parsed.forward.to_card();
        assert_eq!(fwd_card.reps, card.reps);
        assert_eq!(fwd_card.lapses, card.lapses);
        assert!((fwd_card.stability - card.stability).abs() < 1e-10);
        assert!((fwd_card.difficulty - card.difficulty).abs() < 1e-10);

        // Check reverse last_review
        assert!(parsed.reverse.has_last_review);
        let lr = parsed.reverse.to_last_review_string().unwrap();
        assert!(lr.contains("2026-01-15"));

        // Check forward has no last_review
        assert!(!parsed.forward.has_last_review);
        assert!(parsed.forward.to_last_review_string().is_none());
    }

    #[test]
    fn test_record_file_offset() {
        assert_eq!(FsrsRecord::file_offset(0), 16);
        assert_eq!(FsrsRecord::file_offset(1), 176);
        assert_eq!(FsrsRecord::file_offset(100), 16 + 100 * 160);
    }

    #[test]
    fn test_dir_state_size() {
        // Ensure our constant matches actual write size
        let state = BinaryDirState::from_card_and_review(&Card::new(), &None);
        let mut buf = [0u8; DIR_STATE_SIZE];
        state.write_to(&mut buf);
        // If this compiles and doesn't panic, the size is correct
    }

    #[test]
    fn test_is_due() {
        let mut card = Card::new();
        // New card — due is now, should be due
        let state = BinaryDirState::from_card_and_review(&card, &None);
        assert!(state.is_due());

        // Set due far in the future
        card.due = Utc::now() + chrono::Duration::days(30);
        let state = BinaryDirState::from_card_and_review(&card, &None);
        assert!(!state.is_due());
    }

    #[test]
    fn test_word_text_postcard_roundtrip() {
        let wt = WordText {
            en: "bat".to_string(),
            ru: "летучая мышь".to_string(),
            examples: vec!["The bat flew.".to_string()],
            created_at: "2026-01-01T00:00:00Z".to_string(),
        };
        let bytes = postcard::to_allocvec(&wt).unwrap();
        let parsed: WordText = postcard::from_bytes(&bytes).unwrap();
        assert_eq!(parsed.en, "bat");
        assert_eq!(parsed.ru, "летучая мышь");
        assert_eq!(parsed.examples.len(), 1);
    }
}
