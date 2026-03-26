use std::sync::mpsc::{Receiver, Sender, TryRecvError};
use std::ffi::CStr;

use esp_idf_hal::delay::FreeRtos;
use esp_idf_hal::task::thread::ThreadSpawnConfiguration;
use esp_idf_hal::cpu::Core;

use crate::cardputer_hal::cardputer_hal::CardputerHal;
use crate::screen::{Renderable, Screen};
use crate::types::{Command, Msg, SharedState, SharedStateUpdate, TaskHandle};

pub struct Runtime {
    screen: Box<dyn Screen>,
    hal: &'static mut CardputerHal<'static>,
    shared: SharedState,
    msg_rx: Receiver<Msg>,
    msg_tx: Sender<Msg>,
    state_tx: Sender<Box<dyn Renderable>>,
}

impl Runtime {
    pub fn new(
        screen: Box<dyn Screen>,
        hal: &'static mut CardputerHal<'static>,
        msg_rx: Receiver<Msg>,
        msg_tx: Sender<Msg>,
        state_tx: Sender<Box<dyn Renderable>>,
    ) -> Self {
        Self {
            screen,
            hal,
            shared: SharedState::default(),
            msg_rx,
            msg_tx,
            state_tx,
        }
    }

    pub fn run(mut self) {
        log::info!("runtime: started on Core 1");

        // Mount initial screen
        let cmd = self.screen.on_mount(&mut self.hal, &self.shared, &self.state_tx);
        self.execute(cmd);
        let _ = self.state_tx.send(self.screen.snapshot());

        // Message loop — poll with FreeRtos yield (avoids ESP-IDF pthread issues)
        loop {
            match self.msg_rx.try_recv() {
                Ok(msg) => {
                    let cmd = self.screen.handle_msg(msg, &mut self.hal, &self.shared);
                    self.execute(cmd);
                    let _ = self.state_tx.send(self.screen.snapshot());
                }
                Err(TryRecvError::Empty) => {
                    FreeRtos::delay_ms(1); // yield, 0 CPU when idle
                }
                Err(TryRecvError::Disconnected) => {
                    log::error!("runtime: msg channel disconnected");
                    return;
                }
            }
        }
    }

    fn execute(&mut self, cmd: Command) {
        match cmd {
            Command::None => {}
            Command::SpawnTask { id, task } => {
                let handle = TaskHandle {
                    id,
                    tx: self.msg_tx.clone(),
                };
                ThreadSpawnConfiguration {
                    name: Some(unsafe { CStr::from_bytes_with_nul_unchecked(b"task\0") }),
                    stack_size: 8192,
                    priority: 4,
                    pin_to_core: Some(Core::Core1),
                    ..Default::default()
                }
                .set()
                .ok();
                std::thread::spawn(move || task(handle));
            }
            Command::SwitchTo(mut new_screen) => {
                let mount_cmd = new_screen.on_mount(&mut self.hal, &self.shared, &self.state_tx);
                self.screen = new_screen;
                // Send snapshot after mount so Core 0 can draw the new screen
                let _ = self.state_tx.send(self.screen.snapshot());
                self.execute(mount_cmd);
            }
            Command::UpdateShared(update) => match update {
                SharedStateUpdate::SetSdMounted(v) => self.shared.sd_mounted = v,
            },
            Command::Multiple(cmds) => {
                for c in cmds {
                    self.execute(c);
                }
            }
        }
    }
}
