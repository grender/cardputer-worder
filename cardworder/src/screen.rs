use std::sync::mpsc::Sender;

use crate::types::{Command, Core0Action, Msg, SharedState};
use crate::ui::cardworder_ui::CardworderUi;

/// Concrete snapshot enum — avoids Box<dyn> fat pointer / vtable issues across ESP-IDF threads.
pub enum Snapshot {
    MainMenu(crate::screens::main_menu::MainMenuSnapshot),
    Start(crate::screens::start::StartSnapshot),
    SystemInfo(crate::screens::system_info::SystemInfoSnapshot),
    WifiConfig(crate::screens::wifi_config::WifiConfigSnapshot),
    WifiConnect(crate::screens::wifi_connect::WifiConnectSnapshot),
    Ntp(crate::screens::ntp::NtpSnapshot),
    AddWord(crate::screens::add_word::AddWordSnapshot),
    Statistics(crate::screens::statistics::StatisticsSnapshot),
    Review(crate::screens::review::ReviewSnapshot),
}

impl Snapshot {
    pub fn draw(&self, ui: &mut CardworderUi) {
        match self {
            Snapshot::MainMenu(s) => s.draw(ui),
            Snapshot::Start(s) => s.draw(ui),
            Snapshot::SystemInfo(s) => s.draw(ui),
            Snapshot::WifiConfig(s) => s.draw(ui),
            Snapshot::WifiConnect(s) => s.draw(ui),
            Snapshot::Ntp(s) => s.draw(ui),
            Snapshot::AddWord(s) => s.draw(ui),
            Snapshot::Statistics(s) => s.draw(ui),
            Snapshot::Review(s) => s.draw(ui),
        }
    }

    pub fn needs_top_line(&self) -> bool {
        match self {
            Snapshot::Start(_) => false,
            _ => true,
        }
    }

    pub fn action(&self) -> Option<Core0Action> {
        match self {
            Snapshot::Start(s) => s.pending_action.clone(),
            Snapshot::WifiConfig(s) => s.pending_action.clone(),
            Snapshot::WifiConnect(s) => s.pending_action.clone(),
            Snapshot::Ntp(s) => s.pending_action.clone(),
            Snapshot::SystemInfo(s) => s.pending_action.clone(),
            Snapshot::AddWord(s) => s.pending_action.clone(),
            Snapshot::Statistics(s) => s.pending_action.clone(),
            Snapshot::Review(s) => s.pending_action.clone(),
            _ => None,
        }
    }
}

/// Implemented on the Core 1 side. One screen is active at a time.
/// Screens do NOT get HAL access — use Core0Action via snapshots instead.
pub trait Screen: Send {
    /// Called once when this screen becomes active (Core 1).
    fn on_mount(
        &mut self,
        _shared: &SharedState,
        _state_tx: &Sender<Snapshot>,
    ) -> Command {
        Command::None
    }

    /// Core 1: process a message, mutate self, return a command.
    fn handle_msg(&mut self, msg: Msg, shared: &SharedState) -> Command;

    /// Core 1: produce a snapshot for Core 0 to draw.
    fn snapshot(&self, shared: &SharedState) -> Snapshot;
}
