//! SimExecutor: implements all domain executor traits for the desktop simulator.
//! WiFi and NTP operations are stubbed with alternating success/error behaviour.

use std::sync::mpsc::Sender;
use std::thread;
use std::time::{Duration, Instant};

use cardworder_core::types::{
    Core0Result, Msg, NtpExecutor, QuickStats, ScannedNetwork,
    StorageExecutor, SystemExecutor, WifiConfig, WifiConfigList, WifiExecutor,
};

use crate::storage::SimStorage;

pub struct SimExecutor {
    storage: SimStorage,
    wifi_call_count: u32,
    ntp_call_count: u32,
    ntp_started_at: Option<Instant>,
    msg_tx: Sender<Msg>,
    /// Postcard-serialised wifi list cached in the data dir
    wifi_bin_path: std::path::PathBuf,
}

impl SimExecutor {
    pub fn new(msg_tx: Sender<Msg>) -> Self {
        let storage = SimStorage::new();
        let base = dirs::data_dir()
            .or_else(dirs::config_dir)
            .unwrap_or_else(|| std::path::PathBuf::from("."));
        let wifi_bin_path = base.join("cardworder").join("WIFI.BIN");
        SimExecutor {
            storage,
            wifi_call_count: 0,
            ntp_call_count: 0,
            ntp_started_at: None,
            msg_tx,
            wifi_bin_path,
        }
    }

    fn wifi_success(&mut self) -> bool {
        self.wifi_call_count += 1;
        self.wifi_call_count % 2 == 1
    }

    fn ntp_success(&mut self) -> bool {
        self.ntp_call_count += 1;
        self.ntp_call_count % 2 == 1
    }
}

// ── SystemExecutor ────────────────────────────────────────────────────────────

impl SystemExecutor for SimExecutor {
    fn set_timezone(&mut self) -> Result<(), String> {
        Ok(())
    }

    fn get_network_info(&mut self) -> Result<heapless::String<16>, String> {
        let mut s = heapless::String::new();
        s.push_str("192.168.1.1").ok();
        Ok(s)
    }

    fn read_battery(&mut self) -> Result<(u32, u8), String> {
        Ok((4200, 100))
    }
}

// ── WifiExecutor ──────────────────────────────────────────────────────────────

impl WifiExecutor for SimExecutor {
    fn load_wifi_list(&mut self) -> Result<WifiConfigList, String> {
        if self.wifi_bin_path.exists() {
            let bytes = std::fs::read(&self.wifi_bin_path).map_err(|e| e.to_string())?;
            postcard::from_bytes(&bytes).map_err(|e| e.to_string())
        } else {
            Ok(WifiConfigList::default())
        }
    }

    fn save_wifi_list(&mut self, list: WifiConfigList) -> Result<(), String> {
        let bytes = postcard::to_allocvec(&list).map_err(|e| e.to_string())?;
        std::fs::write(&self.wifi_bin_path, &bytes).map_err(|e| e.to_string())
    }

    fn start_wifi(&mut self) -> Result<(), String> {
        let success = self.wifi_success();
        let tx = self.msg_tx.clone();
        thread::spawn(move || {
            thread::sleep(Duration::from_millis(1000));
            let result = if success {
                let mut ssid = heapless::String::new();
                ssid.push_str("SimNet").ok();
                Core0Result::WifiConnected { ssid }
            } else {
                Core0Result::Error("WiFi: connection timeout".into())
            };
            let _ = tx.send(Msg::Core0Result(result));
        });
        Ok(())
    }

    fn start_wifi_scan(&mut self) -> Result<Vec<ScannedNetwork>, String> {
        let networks = vec![
            ScannedNetwork {
                ssid: {
                    let mut s = heapless::String::new();
                    s.push_str("SimNet").ok();
                    s
                },
                signal_strength: -45,
                has_saved_password: false,
            },
            ScannedNetwork {
                ssid: {
                    let mut s = heapless::String::new();
                    s.push_str("HomeWifi").ok();
                    s
                },
                signal_strength: -65,
                has_saved_password: false,
            },
            ScannedNetwork {
                ssid: {
                    let mut s = heapless::String::new();
                    s.push_str("GuestNet").ok();
                    s
                },
                signal_strength: -80,
                has_saved_password: false,
            },
        ];
        Ok(networks)
    }

    fn connect_wifi(&mut self, config: WifiConfig) -> Result<heapless::String<32>, String> {
        let success = self.wifi_success();
        let tx = self.msg_tx.clone();
        let ssid = config.ssid.clone();
        let ssid2 = ssid.clone();
        thread::spawn(move || {
            thread::sleep(Duration::from_millis(800));
            let result = if success {
                Core0Result::WifiConnected { ssid }
            } else {
                Core0Result::Error("WiFi: connect failed".into())
            };
            let _ = tx.send(Msg::Core0Result(result));
        });
        Ok(ssid2)
    }

    fn stop_wifi(&mut self) -> Result<(), String> {
        Ok(())
    }

    fn create_wifi_file_if_not_exists(
        &mut self,
        ssid: heapless::String<32>,
        password: heapless::String<64>,
    ) -> Result<(), String> {
        if !self.wifi_bin_path.exists() {
            let mut list = WifiConfigList::default();
            list.configs.push(WifiConfig { ssid, password });
            self.save_wifi_list(list)?;
        }
        Ok(())
    }

    fn load_wifi_config(&mut self) -> Result<WifiConfig, String> {
        let list = self.load_wifi_list()?;
        list.configs
            .into_iter()
            .next()
            .ok_or_else(|| "No saved WiFi config".into())
    }
}

// ── NtpExecutor ───────────────────────────────────────────────────────────────

impl NtpExecutor for SimExecutor {
    fn start_ntp(&mut self) -> Result<(), String> {
        self.ntp_started_at = Some(Instant::now());
        Ok(())
    }

    fn check_ntp_status(&mut self) -> Result<bool, String> {
        let started = match self.ntp_started_at {
            Some(t) => t,
            None => return Ok(true),
        };
        let success = self.ntp_success();
        let elapsed = started.elapsed();
        if success {
            if elapsed >= Duration::from_millis(1500) {
                self.ntp_started_at = None;
                Ok(true)
            } else {
                Ok(false)
            }
        } else {
            if elapsed >= Duration::from_millis(2000) {
                self.ntp_started_at = None;
                Err("NTP: sync timeout".into())
            } else {
                Ok(false)
            }
        }
    }
}

// ── StorageExecutor ───────────────────────────────────────────────────────────

impl StorageExecutor for SimExecutor {
    fn load_due_items(
        &mut self,
    ) -> Result<(Vec<fsrs_core::DueItem>, Vec<fsrs_core::DueItem>, u64), String> {
        self.storage.load_due_items()
    }

    fn load_word_text(
        &mut self,
        _slot: usize,
        word_offset: u32,
        word_length: u32,
    ) -> Result<(String, String, [u8; 160]), String> {
        let wt = self.storage.load_word_text(word_offset, word_length)?;
        let rec = self.storage.load_fsrs_record(_slot)?;
        Ok((wt.en, wt.ru, rec.to_bytes()))
    }

    fn rate_card(&mut self, slot: usize, record_bytes: [u8; 160]) -> Result<(), String> {
        let record = fsrs_core::FsrsRecord::from_bytes(&record_bytes);
        self.storage.save_fsrs_record(slot, &record)
    }

    fn rate_and_load_next(
        &mut self,
        slot: usize,
        record_bytes: [u8; 160],
        next_slot: usize,
        next_word_offset: u32,
        next_word_length: u32,
    ) -> Result<(String, String, [u8; 160]), String> {
        let record = fsrs_core::FsrsRecord::from_bytes(&record_bytes);
        self.storage.save_fsrs_record(slot, &record)?;
        let wt = self.storage.load_word_text(next_word_offset, next_word_length)?;
        let rec = self.storage.load_fsrs_record(next_slot)?;
        Ok((wt.en, wt.ru, rec.to_bytes()))
    }

    fn add_pair(&mut self, en: String, ru: String) -> Result<u64, String> {
        self.storage.add_pair(&en, &ru)
    }

    fn load_quick_stats(&mut self) -> Result<QuickStats, String> {
        self.storage.load_quick_stats()
    }

    fn load_next_id(&mut self) -> Result<u64, String> {
        self.storage.load_next_id()
    }

    fn migrate_pairs(&mut self) -> Result<(), String> {
        self.storage.migrate_pairs()
    }
}
