use std::sync::mpsc::Sender;

use crate::input::keyboard::{InputLanguage, InputState, PressedSymbol};
use crate::input::keyboard_io::{KeyEvent, Scancode};
use crate::screen::Screen;

// ── WiFi data types ─────────────────────────────────────────────────────────

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct WifiConfig {
    pub ssid: heapless::String<32>,
    pub password: heapless::String<64>,
}

/// WiFi config list stored on SD card.
#[derive(Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct WifiConfigList {
    pub configs: Vec<WifiConfig>,
}

/// Scanned WiFi network info.
#[derive(Clone)]
pub struct ScannedNetwork {
    pub ssid: heapless::String<32>,
    pub signal_strength: i8,
    pub has_saved_password: bool,
}

// ── Storage statistics ───────────────────────────────────────────────────────

/// Per-direction FSRS statistics.
#[derive(Clone, Default)]
pub struct DirStats {
    pub due: usize,
    pub new_count: usize,
    pub learning: usize,
    pub mastered: usize,
    pub total_reviews: i64,
    pub total_lapses: i64,
    pub avg_difficulty: f32,
    pub hardest_word: Option<String>,
    pub hardest_lapses: i32,
    pub strongest_word: Option<String>,
    pub strongest_days: i64,
    pub weakest_word: Option<String>,
    pub weakest_days: i64,
}

/// Compact statistics computed from FSRS.BIN.
#[derive(Clone)]
pub struct QuickStats {
    pub total_pairs: usize,
    pub forward: DirStats,
    pub reverse: DirStats,
}

// ── Core 0 actions and results ───────────────────────────────────────────────

pub enum Core0Action {
    // System
    SetTimezone,
    GetNetworkInfo,
    ReadBattery,
    // WiFi
    LoadWifiList,
    SaveWifiList(WifiConfigList),
    StartWifi,
    StartWifiScan,
    ConnectWifi(WifiConfig),
    StopWifi,
    CreateWifiFileIfNotExists {
        ssid: heapless::String<32>,
        password: heapless::String<64>,
    },
    LoadWifiConfig,
    // NTP
    StartNtp,
    CheckNtpStatus,
    // Storage
    LoadDueItems,
    LoadWordText { slot: usize, word_offset: u32, word_length: u32 },
    RateCard { slot: usize, record_bytes: [u8; 160] },
    RateAndLoadNext {
        slot: usize,
        record_bytes: [u8; 160],
        next_slot: usize,
        next_word_offset: u32,
        next_word_length: u32,
    },
    AddPair { en: String, ru: String },
    LoadQuickStats,
    LoadNextId,
    MigratePairs,
}

pub enum Core0Result {
    // System
    TimezoneSet,
    NetworkInfo { ip: heapless::String<16> },
    BatteryReading { mv: u32, percent: u8 },
    // WiFi
    WifiListLoaded(WifiConfigList),
    WifiListSaved,
    WifiStarted,
    WifiScanResults(Vec<ScannedNetwork>),
    WifiConnected { ssid: heapless::String<32> },
    WifiStopped,
    WifiFileCreated,
    WifiConfigLoaded(WifiConfig),
    // NTP
    NtpStarted,
    NtpSynced(bool),
    // Storage
    DueItemsLoaded {
        forward_items: Vec<fsrs_core::DueItem>,
        reverse_items: Vec<fsrs_core::DueItem>,
        next_id: u64,
    },
    WordTextLoaded { en: String, ru: String, record_bytes: [u8; 160] },
    CardRated,
    CardRatedAndNextLoaded { en: String, ru: String, record_bytes: [u8; 160] },
    PairAdded(u64),
    QuickStatsLoaded(QuickStats),
    NextIdLoaded(u64),
    MigrationDone,
    // Any action can fail
    Error(String),
}

// ── Domain executor traits ───────────────────────────────────────────────────

pub trait SystemExecutor {
    fn set_timezone(&mut self) -> Result<(), String>;
    fn get_network_info(&mut self) -> Result<heapless::String<16>, String>;
    fn read_battery(&mut self) -> Result<(u32, u8), String>;
}

pub trait WifiExecutor {
    fn load_wifi_list(&mut self) -> Result<WifiConfigList, String>;
    fn save_wifi_list(&mut self, list: WifiConfigList) -> Result<(), String>;
    fn start_wifi(&mut self) -> Result<(), String>;
    fn start_wifi_scan(&mut self) -> Result<Vec<ScannedNetwork>, String>;
    fn connect_wifi(&mut self, config: WifiConfig) -> Result<heapless::String<32>, String>;
    fn stop_wifi(&mut self) -> Result<(), String>;
    fn create_wifi_file_if_not_exists(
        &mut self,
        ssid: heapless::String<32>,
        password: heapless::String<64>,
    ) -> Result<(), String>;
    fn load_wifi_config(&mut self) -> Result<WifiConfig, String>;
}

pub trait NtpExecutor {
    fn start_ntp(&mut self) -> Result<(), String>;
    fn check_ntp_status(&mut self) -> Result<bool, String>;
}

pub trait StorageExecutor {
    fn load_due_items(
        &mut self,
    ) -> Result<(Vec<fsrs_core::DueItem>, Vec<fsrs_core::DueItem>, u64), String>;
    fn load_word_text(
        &mut self,
        slot: usize,
        word_offset: u32,
        word_length: u32,
    ) -> Result<(String, String, [u8; 160]), String>;
    fn rate_card(&mut self, slot: usize, record_bytes: [u8; 160]) -> Result<(), String>;
    fn rate_and_load_next(
        &mut self,
        slot: usize,
        record_bytes: [u8; 160],
        next_slot: usize,
        next_word_offset: u32,
        next_word_length: u32,
    ) -> Result<(String, String, [u8; 160]), String>;
    fn add_pair(&mut self, en: String, ru: String) -> Result<u64, String>;
    fn load_quick_stats(&mut self) -> Result<QuickStats, String>;
    fn load_next_id(&mut self) -> Result<u64, String>;
    fn migrate_pairs(&mut self) -> Result<(), String>;
}

pub trait Core0Executor: SystemExecutor + WifiExecutor + NtpExecutor + StorageExecutor {}
impl<T: SystemExecutor + WifiExecutor + NtpExecutor + StorageExecutor> Core0Executor for T {}

/// Type-safe dispatch: single source of truth for action→result mapping.
pub fn dispatch_action(executor: &mut impl Core0Executor, action: Core0Action) -> Core0Result {
    match action {
        Core0Action::SetTimezone => match executor.set_timezone() {
            Ok(()) => Core0Result::TimezoneSet,
            Err(e) => Core0Result::Error(e),
        },
        Core0Action::GetNetworkInfo => match executor.get_network_info() {
            Ok(ip) => Core0Result::NetworkInfo { ip },
            Err(e) => Core0Result::Error(e),
        },
        Core0Action::ReadBattery => match executor.read_battery() {
            Ok((mv, percent)) => Core0Result::BatteryReading { mv, percent },
            Err(e) => Core0Result::Error(e),
        },
        Core0Action::LoadWifiList => match executor.load_wifi_list() {
            Ok(list) => Core0Result::WifiListLoaded(list),
            Err(e) => Core0Result::Error(e),
        },
        Core0Action::SaveWifiList(list) => match executor.save_wifi_list(list) {
            Ok(()) => Core0Result::WifiListSaved,
            Err(e) => Core0Result::Error(e),
        },
        Core0Action::StartWifi => match executor.start_wifi() {
            Ok(()) => Core0Result::WifiStarted,
            Err(e) => Core0Result::Error(e),
        },
        Core0Action::StartWifiScan => match executor.start_wifi_scan() {
            Ok(networks) => Core0Result::WifiScanResults(networks),
            Err(e) => Core0Result::Error(e),
        },
        Core0Action::ConnectWifi(config) => match executor.connect_wifi(config) {
            Ok(ssid) => Core0Result::WifiConnected { ssid },
            Err(e) => Core0Result::Error(e),
        },
        Core0Action::StopWifi => match executor.stop_wifi() {
            Ok(()) => Core0Result::WifiStopped,
            Err(e) => Core0Result::Error(e),
        },
        Core0Action::CreateWifiFileIfNotExists { ssid, password } => {
            match executor.create_wifi_file_if_not_exists(ssid, password) {
                Ok(()) => Core0Result::WifiFileCreated,
                Err(e) => Core0Result::Error(e),
            }
        }
        Core0Action::LoadWifiConfig => match executor.load_wifi_config() {
            Ok(config) => Core0Result::WifiConfigLoaded(config),
            Err(e) => Core0Result::Error(e),
        },
        Core0Action::StartNtp => match executor.start_ntp() {
            Ok(()) => Core0Result::NtpStarted,
            Err(e) => Core0Result::Error(e),
        },
        Core0Action::CheckNtpStatus => match executor.check_ntp_status() {
            Ok(done) => Core0Result::NtpSynced(done),
            Err(e) => Core0Result::Error(e),
        },
        Core0Action::LoadDueItems => match executor.load_due_items() {
            Ok((forward_items, reverse_items, next_id)) => {
                Core0Result::DueItemsLoaded { forward_items, reverse_items, next_id }
            }
            Err(e) => Core0Result::Error(e),
        },
        Core0Action::LoadWordText { slot, word_offset, word_length } => {
            match executor.load_word_text(slot, word_offset, word_length) {
                Ok((en, ru, record_bytes)) => Core0Result::WordTextLoaded { en, ru, record_bytes },
                Err(e) => Core0Result::Error(e),
            }
        }
        Core0Action::RateCard { slot, record_bytes } => match executor.rate_card(slot, record_bytes) {
            Ok(()) => Core0Result::CardRated,
            Err(e) => Core0Result::Error(e),
        },
        Core0Action::RateAndLoadNext { slot, record_bytes, next_slot, next_word_offset, next_word_length } => {
            match executor.rate_and_load_next(slot, record_bytes, next_slot, next_word_offset, next_word_length) {
                Ok((en, ru, record_bytes)) => Core0Result::CardRatedAndNextLoaded { en, ru, record_bytes },
                Err(e) => Core0Result::Error(e),
            }
        }
        Core0Action::AddPair { en, ru } => match executor.add_pair(en, ru) {
            Ok(id) => Core0Result::PairAdded(id),
            Err(e) => Core0Result::Error(e),
        },
        Core0Action::LoadQuickStats => match executor.load_quick_stats() {
            Ok(stats) => Core0Result::QuickStatsLoaded(stats),
            Err(e) => Core0Result::Error(e),
        },
        Core0Action::LoadNextId => match executor.load_next_id() {
            Ok(id) => Core0Result::NextIdLoaded(id),
            Err(e) => Core0Result::Error(e),
        },
        Core0Action::MigratePairs => match executor.migrate_pairs() {
            Ok(()) => Core0Result::MigrationDone,
            Err(e) => Core0Result::Error(e),
        },
    }
}

// ── Message / command types ───────────────────────────────────────────────────

#[derive(Clone, Copy)]
pub struct KeyMsg {
    pub key: Option<(KeyEvent, Scancode)>,
    pub input_state: InputState,
    pub pressed: Option<(KeyEvent, PressedSymbol)>,
}

pub enum Msg {
    Key(KeyMsg),
    TaskProgress { id: u8, percent: u8 },
    TaskDone { id: u8 },
    TaskError { id: u8, msg: &'static str },
    Core0Result(Core0Result),
}

pub enum Command {
    None,
    SpawnTask {
        id: u8,
        task: Box<dyn FnOnce(TaskHandle) + Send + 'static>,
    },
    SwitchTo(Box<dyn Screen + Send>),
    UpdateShared(SharedStateUpdate),
    Multiple(Vec<Command>),
}

#[derive(Clone)]
pub struct TaskHandle {
    pub id: u8,
    pub tx: Sender<Msg>,
}

impl TaskHandle {
    pub fn progress(&self, pct: u8) {
        let _ = self.tx.send(Msg::TaskProgress { id: self.id, percent: pct });
    }
    pub fn done(self) {
        let _ = self.tx.send(Msg::TaskDone { id: self.id });
    }
    pub fn error(self, msg: &'static str) {
        let _ = self.tx.send(Msg::TaskError { id: self.id, msg });
    }
}

#[derive(Clone)]
pub struct SharedState {
    pub sd_mounted: bool,
    pub wifi_connected: bool,
    pub wifi_ssid: Option<heapless::String<32>>,
    pub lang: InputLanguage,
}

impl Default for SharedState {
    fn default() -> Self {
        Self {
            sd_mounted: false,
            wifi_connected: false,
            wifi_ssid: None,
            lang: InputLanguage::En,
        }
    }
}

pub enum SharedStateUpdate {
    SetSdMounted(bool),
}
