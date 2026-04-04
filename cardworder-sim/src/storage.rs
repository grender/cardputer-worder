//! File-based storage for the desktop simulator.
//! Mirrors the ESP SD-card layout: FSRS.BIN + WORDS.BIN in the OS config dir.

use std::fs;
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};

use fsrs_core::{
    BinaryDirState, DueItem, Direction, FsrsHeader, FsrsRecord, WordText,
    FSRS_HEADER_SIZE, FSRS_RECORD_SIZE,
};

pub struct SimStorage {
    data_dir: PathBuf,
}

impl SimStorage {
    pub fn new() -> Self {
        let base = dirs::data_dir()
            .or_else(dirs::config_dir)
            .unwrap_or_else(|| PathBuf::from("."));
        let data_dir = base.join("cardworder");
        fs::create_dir_all(&data_dir).ok();
        SimStorage { data_dir }
    }

    fn fsrs_path(&self) -> PathBuf {
        self.data_dir.join("FSRS.BIN")
    }

    fn words_path(&self) -> PathBuf {
        self.data_dir.join("WORDS.BIN")
    }

    fn read_file(&self, path: &Path) -> std::io::Result<Vec<u8>> {
        fs::read(path)
    }

    fn write_file(&self, path: &Path, data: &[u8]) -> std::io::Result<()> {
        fs::write(path, data)
    }

    fn read_at(&self, path: &Path, offset: u32, buf: &mut [u8]) -> std::io::Result<()> {
        let mut f = fs::File::open(path)?;
        f.seek(SeekFrom::Start(offset as u64))?;
        f.read_exact(buf)
    }

    fn write_at(&self, path: &Path, offset: u32, data: &[u8]) -> std::io::Result<()> {
        let mut f = fs::OpenOptions::new().write(true).open(path)?;
        f.seek(SeekFrom::Start(offset as u64))?;
        f.write_all(data)
    }

    fn append(&self, path: &Path, data: &[u8]) -> std::io::Result<u32> {
        let mut f = fs::OpenOptions::new().create(true).append(true).open(path)?;
        let pos = f.seek(SeekFrom::End(0))? as u32;
        f.write_all(data)?;
        Ok(pos)
    }

    // ---- Public storage operations ----

    pub fn load_next_id(&self) -> Result<u64, String> {
        let path = self.fsrs_path();
        if !path.exists() {
            return Ok(1);
        }
        let bytes = self.read_file(&path).map_err(|e| e.to_string())?;
        if bytes.len() < FSRS_HEADER_SIZE {
            return Ok(1);
        }
        let header = FsrsHeader::from_bytes(bytes[..FSRS_HEADER_SIZE].try_into().unwrap())
            .ok_or_else(|| "Invalid FSRS header".to_string())?;
        Ok(header.next_id)
    }

    pub fn load_due_items(
        &self,
    ) -> Result<(Vec<DueItem>, Vec<DueItem>, u64), String> {
        let path = self.fsrs_path();
        if !path.exists() {
            return Ok((Vec::new(), Vec::new(), 1));
        }
        let bytes = self.read_file(&path).map_err(|e| e.to_string())?;
        if bytes.len() < FSRS_HEADER_SIZE {
            return Ok((Vec::new(), Vec::new(), 1));
        }
        let header = FsrsHeader::from_bytes(bytes[..FSRS_HEADER_SIZE].try_into().unwrap())
            .ok_or_else(|| "Invalid FSRS header".to_string())?;

        let record_data = &bytes[FSRS_HEADER_SIZE..];
        let num_records = record_data.len() / FSRS_RECORD_SIZE;
        let mut forward = Vec::new();
        let mut reverse = Vec::new();

        for slot in 0..num_records {
            let start = slot * FSRS_RECORD_SIZE;
            let rec_bytes: &[u8; FSRS_RECORD_SIZE] =
                record_data[start..start + FSRS_RECORD_SIZE].try_into().unwrap();
            let rec = FsrsRecord::from_bytes(rec_bytes);
            if rec.forward.is_due() {
                forward.push(DueItem {
                    pair_id: rec.pair_id,
                    direction: Direction::Forward,
                    slot,
                    word_offset: rec.word_offset,
                    word_length: rec.word_length,
                });
            }
            if rec.reverse.is_due() {
                reverse.push(DueItem {
                    pair_id: rec.pair_id,
                    direction: Direction::Reverse,
                    slot,
                    word_offset: rec.word_offset,
                    word_length: rec.word_length,
                });
            }
        }
        Ok((forward, reverse, header.next_id))
    }

    pub fn load_word_text(
        &self,
        word_offset: u32,
        word_length: u32,
    ) -> Result<WordText, String> {
        let path = self.words_path();
        let mut buf = vec![0u8; word_length as usize];
        self.read_at(&path, word_offset, &mut buf).map_err(|e| e.to_string())?;
        if buf.len() < 2 {
            return Err("Word entry too short".into());
        }
        let entry_len = u16::from_le_bytes([buf[0], buf[1]]) as usize;
        let data = &buf[2..2 + entry_len.min(buf.len() - 2)];
        postcard::from_bytes(data).map_err(|e| e.to_string())
    }

    pub fn load_fsrs_record(&self, slot: usize) -> Result<FsrsRecord, String> {
        let path = self.fsrs_path();
        let mut buf = [0u8; FSRS_RECORD_SIZE];
        let offset = FsrsRecord::file_offset(slot);
        self.read_at(&path, offset, &mut buf).map_err(|e| e.to_string())?;
        Ok(FsrsRecord::from_bytes(&buf))
    }

    pub fn save_fsrs_record(&self, slot: usize, record: &FsrsRecord) -> Result<(), String> {
        let path = self.fsrs_path();
        let offset = FsrsRecord::file_offset(slot);
        let bytes = record.to_bytes();
        self.write_at(&path, offset, &bytes).map_err(|e| e.to_string())
    }

    pub fn add_pair(&self, en: &str, ru: &str) -> Result<u64, String> {
        let fsrs_path = self.fsrs_path();
        let words_path = self.words_path();

        let (next_id, mut fsrs_bytes) = if fsrs_path.exists() {
            let bytes = self.read_file(&fsrs_path).map_err(|e| e.to_string())?;
            if bytes.len() < FSRS_HEADER_SIZE {
                let header = FsrsHeader::new(1);
                (1u64, header.to_bytes().to_vec())
            } else {
                let header = FsrsHeader::from_bytes(bytes[..FSRS_HEADER_SIZE].try_into().unwrap())
                    .ok_or_else(|| "Invalid header".to_string())?;
                let id = header.next_id;
                (id, bytes)
            }
        } else {
            let header = FsrsHeader::new(1);
            (1u64, header.to_bytes().to_vec())
        };

        let pair_id = next_id;

        let wt = WordText {
            en: en.to_string(),
            ru: ru.to_string(),
            examples: Vec::new(),
            created_at: chrono::Utc::now().to_rfc3339(),
        };
        let wt_bytes = postcard::to_allocvec(&wt).map_err(|e| e.to_string())?;
        let mut word_entry = Vec::with_capacity(2 + wt_bytes.len());
        word_entry.extend_from_slice(&(wt_bytes.len() as u16).to_le_bytes());
        word_entry.extend_from_slice(&wt_bytes);

        // Append word text, get its offset
        let word_offset = self.append(&words_path, &word_entry).map_err(|e| e.to_string())?;
        let word_length = word_entry.len() as u32;

        // Build FSRS record with new card states
        let new_card = rs_fsrs::Card::new();
        let dir_state = BinaryDirState::from_card_and_review(&new_card, &None);
        let record = FsrsRecord {
            pair_id,
            word_offset,
            word_length,
            forward: dir_state.clone(),
            reverse: dir_state,
        };

        // Append record to fsrs_bytes and update header
        fsrs_bytes.extend_from_slice(&record.to_bytes());
        let new_header = FsrsHeader::new(pair_id + 1);
        fsrs_bytes[..FSRS_HEADER_SIZE].copy_from_slice(&new_header.to_bytes());
        self.write_file(&fsrs_path, &fsrs_bytes).map_err(|e| e.to_string())?;

        Ok(pair_id)
    }

    pub fn load_quick_stats(&self) -> Result<cardworder_core::types::QuickStats, String> {
        use cardworder_core::types::{DirStats, QuickStats};

        let empty = || QuickStats { total_pairs: 0, forward: DirStats::default(), reverse: DirStats::default() };
        let fsrs_path = self.fsrs_path();
        if !fsrs_path.exists() {
            return Ok(empty());
        }
        let bytes = self.read_file(&fsrs_path).map_err(|e| e.to_string())?;
        if bytes.len() < FSRS_HEADER_SIZE {
            return Ok(empty());
        }
        let record_data = &bytes[FSRS_HEADER_SIZE..];
        let num_records = record_data.len() / FSRS_RECORD_SIZE;
        if num_records == 0 {
            return Ok(empty());
        }

        let mut fwd = DirStats::default();
        let mut rev = DirStats::default();

        for i in 0..num_records {
            let start = i * FSRS_RECORD_SIZE;
            let rec_bytes: &[u8; FSRS_RECORD_SIZE] =
                record_data[start..start + FSRS_RECORD_SIZE].try_into().unwrap();
            let rec = FsrsRecord::from_bytes(rec_bytes);

            for (ds, dir_state) in [(&mut fwd, &rec.forward), (&mut rev, &rec.reverse)] {
                ds.total_reviews += dir_state.reps as i64;
                ds.total_lapses += dir_state.lapses as i64;
                ds.avg_difficulty += dir_state.difficulty as f32;
                match dir_state.state {
                    0 => ds.new_count += 1,
                    1 | 3 => ds.learning += 1,
                    2 => {
                        if dir_state.scheduled_days >= 21 {
                            ds.mastered += 1;
                        }
                    }
                    _ => {}
                }
                if dir_state.is_due() {
                    ds.due += 1;
                }
            }
        }

        if num_records > 0 {
            fwd.avg_difficulty /= num_records as f32;
            rev.avg_difficulty /= num_records as f32;
        }

        Ok(QuickStats { total_pairs: num_records, forward: fwd, reverse: rev })
    }

    pub fn migrate_pairs(&self) -> Result<(), String> {
        // Migration is from old PAIRS.BIN format — not needed for desktop sim
        Ok(())
    }
}
