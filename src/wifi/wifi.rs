use esp_idf_svc::sd;
use esp_idf_svc::wifi::{EspWifi, ClientConfiguration, Configuration};
use esp_idf_svc::nvs::EspDefaultNvsPartition;
use esp_idf_hal::peripheral::Peripheral;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use heapless::String;
use core::str::FromStr;
use crate::sd::cardputer_sd::CardputerSd;
use esp_idf_hal::delay::Delay;

#[derive(Debug, Serialize, Deserialize)]
pub struct WifiConfig {
    pub ssid: String::<32>,
    pub password: String::<64>,
}
pub struct CardWorderWifi<'a> {
    driver:  EspWifi<'a>,
    sd_card: CardputerSd<'a, Delay>,
}

impl<'a> CardWorderWifi<'a> {
    pub fn new(wifi: EspWifi<'a>, sd_card: CardputerSd<'a, Delay>) -> Self {
        Self { 
            driver: wifi,
            sd_card,
        }
    }
    
    pub fn create_file_if_non_exists(&mut self, ssid: String<32>, password: String<64>) -> Result<()> {
        let is_file_exists = {
            self.sd_card.is_file_exists("wifi_cfg.jsn").unwrap()
        };
        if !is_file_exists {
            let config = WifiConfig { ssid, password };
            let config_str = serde_json::to_string(&config).unwrap();
            self.sd_card.write_file("wifi_cfg.jsn", &config_str).unwrap();
        }
        Ok(())
    }

    pub fn connect(&mut self) -> Result<()> {
        let config_str = self.sd_card.read_file("wifi_cfg.jsn").map_err(|e| {
            anyhow::anyhow!("Failed to read wifi_cfg.jsn")
        })?;

        let config: WifiConfig = serde_json::from_str(&config_str)?;

        let wifi_configuration = ClientConfiguration {
            ssid: heapless::String::try_from("ATOM").unwrap(), //config.ssid,
            password: heapless::String::try_from("pw!!ATOM2023@@").unwrap(), // config.password,
            ..Default::default()
        };

        let client_configuration = Configuration::Client(wifi_configuration);

        self.driver.set_configuration(&client_configuration)?;
        self.driver.start()?;
        self.driver.connect()?;


        while !self.driver.is_connected()? {
            std::thread::sleep(std::time::Duration::from_millis(100));
        }

        log::info!("Connected to WiFi network");
        Ok(())
    }
}