use std::sync::mpsc::Sender;

use embedded_graphics::pixelcolor::Rgb565;
use embedded_graphics::prelude::RgbColor;
use esp_idf_hal::delay::FreeRtos;
use esp_idf_svc::sntp::{EspSntp, SyncStatus};
use esp_idf_sys::{setenv, tzset};

use crate::cardputer_hal::cardputer_hal::CardputerHal;
use crate::screen::{Renderable, Screen};
use crate::screens::main_menu::MainMenuScreen;
use crate::types::{Command, Msg, SharedState};
use crate::ui::cardworder_ui::CardworderUi;
use crate::ResultExt;

pub struct StartScreen {}

impl StartScreen {
    pub fn new() -> Self {
        Self {}
    }

    fn send_status(state_tx: &Sender<Box<dyn Renderable>>, text: &'static str) {
        let _ = state_tx.send(Box::new(StartSnapshot { status_text: text }));
    }
}

impl Screen for StartScreen {
    fn on_mount(
        &mut self,
        hal: &mut CardputerHal<'_>,
        _shared: &SharedState,
        state_tx: &Sender<Box<dyn Renderable>>,
    ) -> Command {
        Self::send_status(state_tx, "Starting...");

        unsafe {
            let env_tz = b"TZ\0";
            let tz = b"GMT-3\0";
            setenv(env_tz.as_ptr() as *const u8, tz.as_ptr() as *const u8, 1);
            tzset();
        }

        Self::send_status(state_tx, "Creating WiFi config...");
        hal.create_wifi_file_if_non_exists(
            heapless::String::try_from("John24").unwrap(),
            heapless::String::try_from("52525252").unwrap(),
        )
        .unwrap_or_log("error create wifi file");

        let wifi_config = hal.load_wifi_config().unwrap_or_log("error load wifi config");

        Self::send_status(state_tx, "Connecting WiFi...");
        hal.connect_wifi(wifi_config).unwrap_or_log("error connecting to wifi");

        Self::send_status(state_tx, "Starting NTP...");
        let ntp = EspSntp::new_default().unwrap();

        Self::send_status(state_tx, "Awaiting NTP...");
        while ntp.get_sync_status() != SyncStatus::Completed {
            FreeRtos::delay_ms(100);
        }

        Self::send_status(state_tx, "Got NTP!");
        hal.stop_wifi().unwrap_or_log("error stopping wifi");

        // Transition back to main menu
        Command::SwitchTo(Box::new(MainMenuScreen::default()))
    }

    fn handle_msg(
        &mut self,
        _msg: Msg,
        _hal: &mut CardputerHal<'_>,
        _shared: &SharedState,
    ) -> Command {
        Command::None
    }

    fn snapshot(&self) -> Box<dyn Renderable> {
        Box::new(StartSnapshot {
            status_text: "Starting...",
        })
    }
}

// ---- Snapshot ----

struct StartSnapshot {
    status_text: &'static str,
}

unsafe impl Send for StartSnapshot {}

impl Renderable for StartSnapshot {
    fn draw(&self, ui: &mut CardworderUi) {
        ui.draw_starting_line_text(self.status_text, Rgb565::BLACK, Rgb565::WHITE);
    }

    fn needs_top_line(&self) -> bool {
        false
    }
}
