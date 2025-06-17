use anyhow::Result;
use esp_idf_svc::wifi::{ClientConfiguration, Configuration, EspWifi};
use esp_idf_sys::usleep;
use heapless::String;
use serde::{Deserialize, Serialize};

use crate::cardputer_hal::sd::cardputer_sd::CardputerSd;

#[derive(Debug, Serialize, Deserialize)]
pub struct WifiConfig {
    pub ssid: String<32>,
    pub password: String<64>,
}
pub struct CardWorderWifi<'a> {
    driver: EspWifi<'a>
}

impl<'a> CardWorderWifi<'a> {
    pub fn new(wifi: EspWifi<'a>) -> Self {
        Self {
            driver: wifi
        }
    }

    pub fn connect(&mut self, wifi_config: WifiConfig) -> Result<()> {

        let wifi_configuration = ClientConfiguration {
            ssid: wifi_config.ssid,
            password: wifi_config.password,
            ..Default::default()
        };

        let client_configuration = Configuration::Client(wifi_configuration);

        self.driver.set_configuration(&client_configuration)?;
        self.driver.start()?;
        self.driver.connect()?;

        while !self.driver.is_connected()? {
            unsafe {
                usleep(1000);
            }
        }

        log::info!("Connected to WiFi network");
        Ok(())
    }
}
