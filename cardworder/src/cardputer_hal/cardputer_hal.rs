use embedded_graphics::{pixelcolor::Rgb565, prelude::WebColors};
use embedded_graphics_framebuf::FrameBuf;
use esp_idf_hal::{delay::Delay, peripherals::Peripherals};
use esp_idf_svc::eventloop::EspSystemEventLoop;
use esp_idf_hal::gpio::{Output, PinDriver, Pull};
use esp_idf_svc::wifi::EspWifi;

use crate::cardputer_hal::{
    input::{keyboard::{InputLanguage, InputState, PressedSymbol}, keyboard_io::{CardputerKeyboard, Scancode, KeyEvent}},
    screen::{cardputer_screen::CardputerScreen, display::CardputerDisplay, framebuffer::CardputerFramebuffer},
    sd::cardputer_sd::CardputerSd,
    wifi::wifi::{CardWorderWifi, WifiConfig}};

#[derive(Clone, Copy)]
pub struct KeyboardState {
    pub key: Option<(KeyEvent, Scancode)>,
    pub input_state: InputState,
    pub pressed: Option<(KeyEvent, PressedSymbol)>,
}

/// All hardware components returned by `build_all`, ready to distribute across tasks.
pub struct CardputerParts<'a> {
    pub keyboard: CardputerKeyboard<'a>,
    pub display: CardputerDisplay<'a>,
    pub framebuffer: FrameBuf<Rgb565, CardputerFramebuffer>,
    pub hal: CardputerHal<'a>,
}

/// Slim HAL that owns only SD + Wi-Fi (used by the app task).
pub struct CardputerHal<'a> {
    sd: CardputerSd<'a, Delay>,
    wifi: CardWorderWifi<'a>,
}

impl<'a> CardputerHal<'a> {
    /// Build all hardware and return parts for distribution to tasks.
    pub fn build_all(
        peripherals: Peripherals,
        sysloop: EspSystemEventLoop,
    ) -> CardputerParts<'a> {
        log::info!("hal: build_all — start");

        log::info!("hal: 3a CardputerScreen::build (SPI2 + display) …");
        let screen = CardputerScreen::build(
            Rgb565::CSS_BLACK,
            peripherals.spi2,
            peripherals.pins.gpio36,
            peripherals.pins.gpio35,
            peripherals.pins.gpio37,
            peripherals.pins.gpio34,
            peripherals.pins.gpio33,
            peripherals.pins.gpio38,
        );
        log::info!("hal: 3a CardputerScreen::build — done");

        log::info!("hal: 3b CardputerSd::build (SPI3 + SD) …");
        let sd = CardputerSd::build(
            peripherals.spi3,
            peripherals.pins.gpio40,
            peripherals.pins.gpio39,
            peripherals.pins.gpio14,
            peripherals.pins.gpio12,
        );
        log::info!("hal: 3b CardputerSd::build — done");

        log::info!("hal: 3c keyboard mux GPIO (8,9,11) …");
        let mux_pins: [PinDriver<'_, Output>; 3] = [
            PinDriver::output(peripherals.pins.gpio8.degrade_output()).unwrap(),
            PinDriver::output(peripherals.pins.gpio9.degrade_output()).unwrap(),
            PinDriver::output(peripherals.pins.gpio11.degrade_output()).unwrap(),
        ];
        log::info!("hal: 3c mux pins — done");

        log::info!("hal: 3d keyboard column GPIO (13,15,3–7) …");
        let column_pins = [
            PinDriver::input(peripherals.pins.gpio13.degrade_input_output(), Pull::Up).unwrap(),
            PinDriver::input(peripherals.pins.gpio15.degrade_input_output(), Pull::Up).unwrap(),
            PinDriver::input(peripherals.pins.gpio3.degrade_input_output(), Pull::Up).unwrap(),
            PinDriver::input(peripherals.pins.gpio4.degrade_input_output(), Pull::Up).unwrap(),
            PinDriver::input(peripherals.pins.gpio5.degrade_input_output(), Pull::Up).unwrap(),
            PinDriver::input(peripherals.pins.gpio6.degrade_input_output(), Pull::Up).unwrap(),
            PinDriver::input(peripherals.pins.gpio7.degrade_input_output(), Pull::Up).unwrap(),
        ];
        log::info!("hal: 3d column pins — done");

        log::info!("hal: 3e CardputerKeyboard::new + init …");
        let mut keyboard = CardputerKeyboard::new(mux_pins, column_pins);
        keyboard.init();
        log::info!("hal: 3e keyboard — done");

        log::info!("hal: 3f EspWifi::new (modem + event loop) …");
        let esp_wifi = EspWifi::new(peripherals.modem, sysloop, None).unwrap();
        log::info!("hal: 3f EspWifi::new — done");

        log::info!("hal: 3g CardWorderWifi::wrap …");
        let wifi = CardWorderWifi::new(esp_wifi);
        log::info!("hal: 3g CardWorderWifi — done");

        let (display, framebuffer) = screen.into_parts();

        log::info!("hal: build_all — complete");
        CardputerParts {
            keyboard,
            display,
            framebuffer,
            hal: CardputerHal { sd, wifi },
        }
    }

    pub fn create_wifi_file_if_non_exists(
        &mut self,
        ssid: heapless::String<32>,
        password: heapless::String<64>,
    ) -> anyhow::Result<()> {
        let is_file_exists = { self.sd.is_file_exists("wifi_cfg.jsn").unwrap() };
        if !is_file_exists {
            let config = WifiConfig { ssid, password };
            let config_str = serde_json::to_string(&config).unwrap();
            self.sd
                .write_file("wifi_cfg.jsn", &config_str)
                .unwrap();
        }
        Ok(())
    }

    pub fn load_wifi_config(&mut self) -> anyhow::Result<WifiConfig> {
        let config_str = self
            .sd
            .read_file("wifi_cfg.jsn")
            .map_err(|_e| anyhow::anyhow!("Failed to read wifi_cfg.jsn"))?;

        let config: WifiConfig = serde_json::from_str(&config_str)?;

        Ok(config)
    }

    pub fn connect_wifi(&mut self, wifi_config: WifiConfig) -> anyhow::Result<()> {
        self.wifi.connect(wifi_config).map_err(|_e| anyhow::anyhow!("Failed to connect to wifi"))
    }

    pub fn stop_wifi(&mut self) -> anyhow::Result<()> {
        self.wifi.stop().map_err(|_e| anyhow::anyhow!("Failed to stop wifi"))
    }
}
