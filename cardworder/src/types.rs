use std::sync::mpsc::Sender;

use crate::cardputer_hal::input::keyboard::{InputState, PressedSymbol};
use crate::cardputer_hal::input::keyboard_io::{KeyEvent, Scancode};
use crate::screen::Screen;

/// Keyboard event snapshot sent from Core 0 to Core 1.
#[derive(Clone, Copy)]
pub struct KeyMsg {
    pub key: Option<(KeyEvent, Scancode)>,
    pub input_state: InputState,
    pub pressed: Option<(KeyEvent, PressedSymbol)>,
}

/// Messages flowing into the Core 1 runtime.
pub enum Msg {
    Key(KeyMsg),
    TaskProgress { id: u8, percent: u8 },
    TaskDone { id: u8 },
    TaskError { id: u8, msg: &'static str },
}

/// Commands returned by `Screen::handle_msg` / `Screen::on_mount`.
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

/// Handle given to background tasks so they can report progress.
#[derive(Clone)]
pub struct TaskHandle {
    pub id: u8,
    pub tx: Sender<Msg>,
}

impl TaskHandle {
    pub fn progress(&self, pct: u8) {
        let _ = self.tx.send(Msg::TaskProgress {
            id: self.id,
            percent: pct,
        });
    }
    pub fn done(self) {
        let _ = self.tx.send(Msg::TaskDone { id: self.id });
    }
    pub fn error(self, msg: &'static str) {
        let _ = self.tx.send(Msg::TaskError { id: self.id, msg });
    }
}

/// State shared across all screens. Only Core 1 mutates it.
#[derive(Clone, Default)]
pub struct SharedState {
    pub sd_mounted: bool,
}

pub enum SharedStateUpdate {
    SetSdMounted(bool),
}
