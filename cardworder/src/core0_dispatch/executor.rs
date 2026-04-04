//! HalExecutor: implements all domain executor traits using CardputerHal + ESP-IDF services.

use core::fmt::Write;

use esp_idf_svc::sntp::{EspSntp, SyncStatus};

use crate::cardputer_hal::cardputer_hal::{BatteryReader, CardputerHal};
use crate::types::{QuickStats, ScannedNetwork, WifiConfig, WifiConfigList};

use super::{NtpExecutor, StorageExecutor, SystemExecutor, WifiExecutor};

/// Holds references to all Core 0 resources needed for action execution.
pub struct HalExecutor<'a, 'hal> {
    pub hal: &'a mut CardputerHal<'hal>,
    pub ntp: &'a mut Option<EspSntp<'static>>,
    pub battery: &'a mut BatteryReader,
}

// ── SystemExecutor ──

impl SystemExecutor for HalExecutor<'_, '_> {
    fn set_timezone(&mut self) -> Result<(), String> {
        unsafe {
            let env_tz = b"TZ\0";
            let tz = b"GMT-3\0";
            esp_idf_sys::setenv(env_tz.as_ptr() as *const u8, tz.as_ptr() as *const u8, 1);
            esp_idf_sys::tzset();
        }
        Ok(())
    }

    fn get_network_info(&mut self) -> Result<heapless::String<16>, String> {
        let mut ip_str = heapless::String::<16>::new();
        unsafe {
            let netif = esp_idf_sys::esp_netif_get_handle_from_ifkey(
                b"WIFI_STA_DEF\0".as_ptr() as *const _,
            );
            if !netif.is_null() {
                let mut ip_info: esp_idf_sys::esp_netif_ip_info_t = core::mem::zeroed();
                if esp_idf_sys::esp_netif_get_ip_info(netif, &mut ip_info) == 0 {
                    let ip = ip_info.ip.addr;
                    if ip != 0 {
                        let _ = write!(
                            &mut ip_str,
                            "{}.{}.{}.{}",
                            ip & 0xFF,
                            (ip >> 8) & 0xFF,
                            (ip >> 16) & 0xFF,
                            (ip >> 24) & 0xFF
                        );
                    }
                }
            }
        }
        Ok(ip_str)
    }

    fn read_battery(&mut self) -> Result<(u32, u8), String> {
        let mv = self.battery.read_mv();
        let percent = self.battery.read_percent();
        Ok((mv, percent))
    }
}

// ── WifiExecutor ──

impl WifiExecutor for HalExecutor<'_, '_> {
    fn load_wifi_list(&mut self) -> Result<WifiConfigList, String> {
        self.hal.load_wifi_list().map_err(|e| format!("Load WiFi list: {:?}", e))
    }

    fn save_wifi_list(&mut self, list: WifiConfigList) -> Result<(), String> {
        self.hal.save_wifi_list(&list).map_err(|e| format!("Save WiFi list: {:?}", e))
    }

    fn start_wifi(&mut self) -> Result<(), String> {
        self.hal.start_wifi().map_err(|e| format!("Start WiFi: {:?}", e))
    }

    fn start_wifi_scan(&mut self) -> Result<Vec<ScannedNetwork>, String> {
        let results = self.hal.scan_wifi().map_err(|e| format!("WiFi scan: {:?}", e))?;
        let networks = results.iter().map(|ap| ScannedNetwork {
            ssid: ap.ssid.clone(),
            signal_strength: ap.signal_strength,
            has_saved_password: false, // set by screen
        }).collect();
        Ok(networks)
    }

    fn connect_wifi(&mut self, config: WifiConfig) -> Result<heapless::String<32>, String> {
        let ssid = config.ssid.clone();
        self.hal.connect_wifi(config).map_err(|e| format!("Connect WiFi: {:?}", e))?;
        Ok(ssid)
    }

    fn stop_wifi(&mut self) -> Result<(), String> {
        self.hal.stop_wifi().map_err(|e| format!("Stop WiFi: {:?}", e))
    }

    fn create_wifi_file_if_not_exists(&mut self, ssid: heapless::String<32>, password: heapless::String<64>) -> Result<(), String> {
        self.hal.create_wifi_file_if_non_exists(ssid, password).map_err(|e| format!("Create wifi file: {:?}", e))
    }

    fn load_wifi_config(&mut self) -> Result<WifiConfig, String> {
        self.hal.load_wifi_config().map_err(|e| format!("Load config: {:?}", e))
    }
}

// ── NtpExecutor ──

impl NtpExecutor for HalExecutor<'_, '_> {
    fn start_ntp(&mut self) -> Result<(), String> {
        match EspSntp::new_default() {
            Ok(sntp) => {
                *self.ntp = Some(sntp);
                Ok(())
            }
            Err(e) => Err(format!("Start NTP: {:?}", e)),
        }
    }

    fn check_ntp_status(&mut self) -> Result<bool, String> {
        if let Some(ref sntp) = self.ntp {
            let done = sntp.get_sync_status() == SyncStatus::Completed;
            if done {
                *self.ntp = None; // drop the SNTP instance
            }
            Ok(done)
        } else {
            Ok(true) // no NTP instance, consider done
        }
    }
}

// ── StorageExecutor ──

impl StorageExecutor for HalExecutor<'_, '_> {
    fn load_due_items(&mut self) -> Result<(Vec<fsrs_core::DueItem>, Vec<fsrs_core::DueItem>, u64), String> {
        self.hal.load_due_items().map_err(|e| format!("Load due: {:?}", e))
    }

    fn load_word_text(&mut self, slot: usize, word_offset: u32, word_length: u32) -> Result<(String, String, [u8; 160]), String> {
        let wt = self.hal.load_word_text(word_offset, word_length).map_err(|e| format!("Load word: {:?}", e))?;
        let rec = self.hal.load_fsrs_record(slot).map_err(|e| format!("Load record: {:?}", e))?;
        Ok((wt.en, wt.ru, rec.to_bytes()))
    }

    fn rate_card(&mut self, slot: usize, record_bytes: [u8; 160]) -> Result<(), String> {
        let record = fsrs_core::FsrsRecord::from_bytes(&record_bytes);
        self.hal.save_fsrs_record(slot, &record).map_err(|e| format!("Rate: {:?}", e))
    }

    fn rate_and_load_next(&mut self, slot: usize, record_bytes: [u8; 160], next_slot: usize, next_word_offset: u32, next_word_length: u32) -> Result<(String, String, [u8; 160]), String> {
        let record = fsrs_core::FsrsRecord::from_bytes(&record_bytes);
        let (wt, rec) = self.hal.rate_and_load_next(slot, &record, next_slot, next_word_offset, next_word_length)
            .map_err(|e| format!("RateAndLoad: {:?}", e))?;
        Ok((wt.en, wt.ru, rec.to_bytes()))
    }

    fn add_pair(&mut self, en: String, ru: String) -> Result<u64, String> {
        self.hal.add_pair(&en, &ru).map_err(|e| format!("Add pair: {:?}", e))
    }

    fn load_quick_stats(&mut self) -> Result<QuickStats, String> {
        self.hal.load_quick_stats().map_err(|e| format!("Stats: {:?}", e))
    }

    fn load_next_id(&mut self) -> Result<u64, String> {
        self.hal.load_next_id().map_err(|e| format!("NextId: {:?}", e))
    }

    fn migrate_pairs(&mut self) -> Result<(), String> {
        self.hal.migrate_pairs_if_needed().map_err(|e| format!("Migrate: {:?}", e))?;
        Ok(())
    }
}
