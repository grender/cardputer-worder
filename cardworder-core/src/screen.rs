use std::sync::mpsc::Sender;

use crate::types::{Command, Core0Action, Msg, SharedState};
use crate::ui::cardworder_ui::CardworderUi;
use crate::ui::framebuffer::CardworderFB;

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
    QuickSync(crate::screens::quick_sync::QuickSyncSnapshot),
}

impl Snapshot {
    pub fn draw<FB: CardworderFB>(&self, ui: &mut CardworderUi<FB>) {
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
            Snapshot::QuickSync(s) => s.draw(ui),
        }
    }

    pub fn needs_top_line(&self) -> bool {
        match self {
            Snapshot::Start(_) => false,
            Snapshot::Review(_) => false,
            _ => true,
        }
    }

    pub fn action(&mut self) -> Option<Core0Action> {
        match self {
            Snapshot::Start(s) => s.pending_action.take(),
            Snapshot::WifiConfig(s) => s.pending_action.take(),
            Snapshot::WifiConnect(s) => s.pending_action.take(),
            Snapshot::Ntp(s) => s.pending_action.take(),
            Snapshot::SystemInfo(s) => s.pending_action.take(),
            Snapshot::AddWord(s) => s.pending_action.take(),
            Snapshot::Statistics(s) => s.pending_action.take(),
            Snapshot::Review(s) => s.pending_action.take(),
            Snapshot::QuickSync(s) => s.pending_action.take(),
            _ => None,
        }
    }
}

pub trait Screen: Send {
    fn on_mount(
        &mut self,
        _shared: &SharedState,
        _state_tx: &Sender<Snapshot>,
    ) -> Command {
        Command::None
    }

    fn handle_msg(&mut self, msg: Msg, shared: &SharedState) -> Command;

    fn snapshot(&self, shared: &SharedState) -> Snapshot;
}
