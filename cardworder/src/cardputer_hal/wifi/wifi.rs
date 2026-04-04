use anyhow::Result;
use esp_idf_svc::wifi::{AccessPointInfo, ClientConfiguration, Configuration, EspWifi};
use esp_idf_hal::delay::FreeRtos;
use heapless::String;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WifiConfig {
    pub ssid: String<32>,
    pub password: String<64>,
}

pub struct CardWorderWifi<'a> {
    driver: EspWifi<'a>,
}

impl<'a> CardWorderWifi<'a> {
    pub fn new(wifi: EspWifi<'a>) -> Self {
        log::info!("wifi: CardWorderWifi::new (driver ready)");
        Self { driver: wifi }
    }

    pub fn start(&mut self) -> Result<()> {
        let config = Configuration::Client(ClientConfiguration::default());
        self.driver.set_configuration(&config)?;
        self.driver.start()?;
        Ok(())
    }

    pub fn scan(&mut self) -> Result<Vec<AccessPointInfo>> {
        let results = self.driver.scan()?;
        Ok(results)
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
            FreeRtos::delay_ms(1);
        }

        log::info!("Connected to WiFi network");

        Ok(())
    }

    pub fn stop(&mut self) -> Result<()> {
        self.driver.stop()?;
        Ok(())
    }
}
