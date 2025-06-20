use embedded_graphics::{pixelcolor::Rgb565, prelude::WebColors};
use esp_idf_hal::{delay::Delay, prelude::Peripherals};
use esp_idf_svc::eventloop::EspSystemEventLoop;
use esp_idf_hal::gpio::{self, IOPin, Output, OutputPin, PinDriver};
use esp_idf_svc::wifi::EspWifi;

use crate::cardputer_hal::{
    input::{keyboard::{InputLanguage, InputState, PressedSymbol}, keyboard_io::{CardputerKeyboard, Scancode, KeyEvent}},
    screen::cardputer_screen::CardputerScreen,
    sd::cardputer_sd::CardputerSd,
    wifi::wifi::{CardWorderWifi, WifiConfig}};

pub struct CardputerHal<'a> {
    screen: Option<CardputerScreen<'a>>,
    sd: CardputerSd<'a, Delay>,
    keyboard: CardputerKeyboard<'a>,
    wifi: CardWorderWifi<'a>,

    pub keyboard_state: KeyboardState,
}

pub struct KeyboardState {
    pub key: Option<(KeyEvent, Scancode)>,
    pub input_state: InputState,
    pub pressed: Option<(KeyEvent,PressedSymbol)>,
}

impl <'a>CardputerHal<'a> {
    pub fn new(peripherals: Peripherals, sysloop: EspSystemEventLoop) -> Self {

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

        let sd = CardputerSd::build(
            peripherals.spi3,
            peripherals.pins.gpio40,
            peripherals.pins.gpio39,
            peripherals.pins.gpio14,
            peripherals.pins.gpio12,
        );

        let mux_pins: [PinDriver<'_, gpio::AnyOutputPin, Output>; 3] = [
            PinDriver::output(peripherals.pins.gpio8.downgrade_output()).unwrap(),
            PinDriver::output(peripherals.pins.gpio9.downgrade_output()).unwrap(),
            PinDriver::output(peripherals.pins.gpio11.downgrade_output()).unwrap(),
        ];
    
        let column_pins = [
            PinDriver::input(peripherals.pins.gpio13.downgrade()).unwrap(),
            PinDriver::input(peripherals.pins.gpio15.downgrade()).unwrap(),
            PinDriver::input(peripherals.pins.gpio3.downgrade()).unwrap(),
            PinDriver::input(peripherals.pins.gpio4.downgrade()).unwrap(),
            PinDriver::input(peripherals.pins.gpio5.downgrade()).unwrap(),
            PinDriver::input(peripherals.pins.gpio6.downgrade()).unwrap(),
            PinDriver::input(peripherals.pins.gpio7.downgrade()).unwrap(),
        ];

        let mut keyboard = CardputerKeyboard::new(mux_pins, column_pins);
        keyboard.init();

        let esp_wifi =
        EspWifi::new(peripherals.modem, sysloop, None).unwrap();

        let wifi = CardWorderWifi::new(esp_wifi);

        let input_state = InputState {
            ctrl_pressed: false,
            shift_pressed: false,
            opt_pressed: false,
            alt_pressed: false,
            fn_pressed: false,
            lang: InputLanguage::En,
        };

        let keyboard_state = KeyboardState {
            key: None,
            input_state,
            pressed: None,
        };

        Self { screen:Some(screen), sd, keyboard, wifi, keyboard_state }
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
            .map_err(|e| anyhow::anyhow!("Failed to read wifi_cfg.jsn"))?;

        let config: WifiConfig = serde_json::from_str(&config_str)?;

        Ok(config)
    }
    
    pub fn connect_wifi(&mut self, wifi_config: WifiConfig) -> anyhow::Result<()> {
        self.wifi.connect(wifi_config).map_err(|e| anyhow::anyhow!("Failed to connect to wifi"))
    }

    pub fn stop_wifi(&mut self) -> anyhow::Result<()> {
        self.wifi.stop().map_err(|e| anyhow::anyhow!("Failed to stop wifi"))
    }

    pub fn take_screen(&mut self) -> CardputerScreen<'a> {
        core::mem::replace(&mut self.screen, None).unwrap()
    }

    pub fn update_keyboard_state(&mut self) {
        let key = self.keyboard.read_events();

        let pressed = match key {
            Some((event, key)) => self.keyboard_state.input_state.eat_keys(event, key).map(|f| (event, f)),
            None => None,
        };
        self.keyboard_state.key = key;
        self.keyboard_state.pressed = pressed;
    }
}