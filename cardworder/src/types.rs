use std::sync::mpsc::Sender;

use crate::cardputer_hal::input::keyboard::{InputState, PressedSymbol};
use crate::cardputer_hal::input::keyboard_io::{KeyEvent, Scancode};
use crate::cardputer_hal::wifi::wifi::WifiConfig;
use crate::screen::Screen;

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
    pub lang: crate::cardputer_hal::input::keyboard::InputLanguage,
}

impl Default for SharedState {
    fn default() -> Self {
        Self {
            sd_mounted: false,
            wifi_connected: false,
            wifi_ssid: None,
            lang: crate::cardputer_hal::input::keyboard::InputLanguage::En,
        }
    }
}

pub enum SharedStateUpdate {
    SetSdMounted(bool),
}

/// WiFi config list stored on SD card as JSON array.
#[derive(Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct WifiConfigList {
    pub configs: Vec<WifiConfig>,
}

/// Scanned WiFi network info (simplified for snapshots).
#[derive(Clone)]
pub struct ScannedNetwork {
    pub ssid: heapless::String<32>,
    pub signal_strength: i8,
    pub has_saved_password: bool,
}

/// Actions that must execute on Core 0 (where SPI/WiFi drivers were initialized).
#[derive(Clone)]
pub enum Core0Action {
    SetTimezone,
    // WiFi config persistence
    LoadWifiList,
    SaveWifiList(WifiConfigList),
    // WiFi scanning
    StartWifi,
    StartWifiScan,
    GetScanResults,
    // WiFi connection
    ConnectWifi(WifiConfig),
    StopWifi,
    // NTP
    StartNtp,
    CheckNtpStatus,
    // Network info (reads IP from esp_netif on Core 0)
    GetNetworkInfo,
    // Legacy (kept for StartScreen compatibility)
    CreateWifiFileIfNotExists {
        ssid: heapless::String<32>,
        password: heapless::String<64>,
    },
    LoadWifiConfig,
}

/// Results of Core0Action, sent back as Msg::Core0Result.
pub enum Core0Result {
    TimezoneSet,
    // WiFi config persistence
    WifiListLoaded(WifiConfigList),
    WifiListSaved,
    // WiFi scanning
    WifiStarted,
    WifiScanStarted,
    WifiScanResults(Vec<ScannedNetwork>),
    // WiFi connection
    WifiConnected { ssid: heapless::String<32> },
    WifiStopped,
    // Network info
    NetworkInfo { ip: heapless::String<16> },
    // NTP
    NtpStarted,
    NtpSynced(bool),
    // Legacy
    WifiFileCreated,
    WifiConfigLoaded(WifiConfig),
    // Error — uses String so dynamic error messages can be shown to user
    Error(String),
}
