use embedded_graphics::{pixelcolor::Rgb565, prelude::RgbColor};
use esp_idf_svc::sntp::{EspSntp, SyncStatus};
use esp_idf_sys::{setenv, tzset};

use crate::{cardputer_hal::{cardputer_hal::{CardputerHal, KeyboardState}, wifi::wifi::WifiConfig}, logic::{view_manager::CardputerView, views::main_menu::MainMenuView}, ui::cardworder_ui::CardworderUi, ResultExt};

pub struct StartView {
}

impl CardputerView for StartView {
    fn is_need_top_line(&self) -> bool {
        false
    }

    fn is_need_clear_on_update(&self) -> bool {
        false
    }

    fn init(&mut self, hal: &mut CardputerHal<'_>, ui: &mut CardworderUi<'_>) {
        ui.clear(Rgb565::BLACK);
        ui.draw_starting_line("Starting...", Rgb565::BLACK, Rgb565::WHITE);
        ui.flip_buffer();

        unsafe {
            let env_tz = b"TZ\0";
            let tz = b"GMT-3\0";
            // TODO: move to a separate file
            setenv(env_tz.as_ptr() as *const i8, tz.as_ptr() as *const i8, 1);
            tzset();
            // let tz = getenv(env_tz.as_ptr() as *const i8);
            // let tz_str = CStr::from_ptr(tz).to_str().unwrap();
            // log::info!("tz: {:?}", tz_str);
        }


        hal.create_wifi_file_if_non_exists(
            heapless::String::try_from("John24").unwrap(),
            heapless::String::try_from("52525252").unwrap(),
        ).unwrap_or_log("error create wifi file");
    
        //let wifi_config = hal.load_wifi_config().unwrap_or_log("error load wifi config");
        let wifi_config = WifiConfig {
            ssid: heapless::String::try_from("ATOM").unwrap(),
            password: heapless::String::try_from("pw!!ATOM2023@@").unwrap(),
        };
        
    
        ui.draw_starting_line("Starting Wifi...", Rgb565::BLACK, Rgb565::WHITE);
        ui.flip_buffer();
    


        hal.connect_wifi(wifi_config).unwrap_or_log("error connecting to wifi");

        ui.draw_starting_line("Starting NTP...", Rgb565::BLACK, Rgb565::WHITE);
        ui.flip_buffer();
    
        let ntp = EspSntp::new_default().unwrap();
    
        ui.draw_starting_line("Awaiting NTP...", Rgb565::BLACK, Rgb565::WHITE);
        ui.flip_buffer();
    
        while ntp.get_sync_status() != SyncStatus::Completed {}
    
        ui.draw_starting_line("Got NTP!", Rgb565::BLACK, Rgb565::WHITE);
        ui.flip_buffer();
    
        hal.stop_wifi().unwrap_or_log("error stopping wifi");
    }

    fn update(&mut self, keyboard_state: &KeyboardState) -> Option<Box<dyn CardputerView>> {
        Some(Box::new(MainMenuView::default()))
    }

    fn draw(&mut self, ui: &mut CardworderUi<'_>) {
    }
}