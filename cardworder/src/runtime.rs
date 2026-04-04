use std::sync::mpsc::{Receiver, Sender, TryRecvError};

use esp_idf_hal::delay::FreeRtos;
use esp_idf_hal::task::thread::ThreadSpawnConfiguration;
use esp_idf_hal::cpu::Core;

use crate::screen::{Snapshot, Screen};
use crate::types::{Command, Msg, SharedState, SharedStateUpdate, TaskHandle};

pub struct Runtime {
    screen: Box<dyn Screen>,
    shared: SharedState,
    msg_rx: Receiver<Msg>,
    msg_tx: Sender<Msg>,
    state_tx: Sender<Snapshot>,
}

impl Runtime {
    pub fn new(
        screen: Box<dyn Screen>,
        msg_rx: Receiver<Msg>,
        msg_tx: Sender<Msg>,
        state_tx: Sender<Snapshot>,
    ) -> Self {
        Self {
            screen,
            shared: SharedState::default(),
            msg_rx,
            msg_tx,
            state_tx,
        }
    }

    pub fn run(mut self) {
        log::info!("runtime: started on Core 1");

        // Mount initial screen
        let cmd = self.screen.on_mount(&self.shared, &self.state_tx);
        self.execute(cmd);
        let _ = self.state_tx.send(self.screen.snapshot(&self.shared));

        // Message loop — poll with FreeRtos yield (avoids ESP-IDF pthread issues)
        loop {
            match self.msg_rx.try_recv() {
                Ok(msg) => {
                    // Update shared state from messages
                    match &msg {
                        Msg::Key(key_msg) => {
                            self.shared.lang = key_msg.input_state.lang;
                        }
                        Msg::Core0Result(ref result) => match result {
                            crate::types::Core0Result::WifiConnected { ref ssid } => {
                                self.shared.wifi_connected = true;
                                self.shared.wifi_ssid = Some(ssid.clone());
                            }
                            crate::types::Core0Result::WifiStopped => {
                                self.shared.wifi_connected = false;
                                self.shared.wifi_ssid = None;
                            }
                            _ => {}
                        },
                        _ => {}
                    }
                    let cmd = self.screen.handle_msg(msg, &self.shared);
                    self.execute(cmd);
                    let _ = self.state_tx.send(self.screen.snapshot(&self.shared));
                }
                Err(TryRecvError::Empty) => {
                    FreeRtos::delay_ms(1);
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
                    name: Some(c"task"),
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
                let mount_cmd = new_screen.on_mount(&self.shared, &self.state_tx);
                self.screen = new_screen;
                let _ = self.state_tx.send(self.screen.snapshot(&self.shared));
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
