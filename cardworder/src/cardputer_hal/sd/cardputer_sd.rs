use embedded_hal::delay::DelayNs;
use embedded_sdmmc::{
    BlockDevice, Directory, Error, Mode, SdCard, TimeSource, VolumeIdx, VolumeManager,
};
use esp_idf_hal::{
    delay::Delay,
    gpio::{InputPin, OutputPin},
    spi::{
        config::DriverConfig, SpiAnyPins, SpiConfig, SpiDeviceDriver, SpiDriver,
    },
    units::FromValueType,
};

use embedded_sdmmc::SdCardError;

pub struct CardputerSd<'a, DELAYER>
where
    DELAYER: DelayNs + 'a,
{
    volume_manager:
        VolumeManager<SdCard<SpiDeviceDriver<'a, SpiDriver<'a>>, DELAYER>, FakeTimesource, 4, 4, 1>,
}

struct FakeTimesource();

impl embedded_sdmmc::TimeSource for FakeTimesource {
    fn get_timestamp(&self) -> embedded_sdmmc::Timestamp {
        embedded_sdmmc::Timestamp {
            year_since_1970: 0,
            zero_indexed_month: 0,
            zero_indexed_day: 0,
            hours: 0,
            minutes: 0,
            seconds: 0,
        }
    }
}

impl CardputerSd<'_, Delay> {
    pub fn build<'a, SPI: SpiAnyPins + 'a>(
        spi: SPI,
        sclk: impl OutputPin + 'a,
        miso: impl InputPin + 'a,
        mosi: impl OutputPin + 'a,
        cs: impl OutputPin + 'a,
    ) -> CardputerSd<'a, Delay> {
        log::info!("sd: CardputerSd::build — start (SPI3)");
        let mut delay = Delay::new_default();

        // After ESP-IDF 5.1→5.5 upgrade, the SPI driver no longer sends the mandatory
        // 74+ clock cycles with CS HIGH that the SD spec requires for power-up.
        // We bit-bang them via raw GPIO before initializing the SPI driver.
        let cs_pin = cs.pin() as i32;
        let sclk_pin = sclk.pin() as i32;
        let mosi_pin = mosi.pin() as i32;

        unsafe {
            // Configure CS, SCLK, MOSI as GPIO outputs, all HIGH
            esp_idf_sys::gpio_set_direction(cs_pin, esp_idf_sys::gpio_mode_t_GPIO_MODE_OUTPUT);
            esp_idf_sys::gpio_set_level(cs_pin, 1);
            esp_idf_sys::gpio_set_direction(sclk_pin, esp_idf_sys::gpio_mode_t_GPIO_MODE_OUTPUT);
            esp_idf_sys::gpio_set_level(sclk_pin, 1);
            esp_idf_sys::gpio_set_direction(mosi_pin, esp_idf_sys::gpio_mode_t_GPIO_MODE_OUTPUT);
            esp_idf_sys::gpio_set_level(mosi_pin, 1); // MOSI high = 0xFF bytes
        }

        // Wait for card power-up (SD spec: 1ms min, some cards need more)
        delay.delay_ms(250);

        // Bit-bang 80 clock cycles on SCLK with CS HIGH (SD spec requirement)
        log::info!("sd: sending 80 init clocks via GPIO bit-bang …");
        unsafe {
            for _ in 0..80 {
                esp_idf_sys::gpio_set_level(sclk_pin, 0);
                esp_idf_sys::esp_rom_delay_us(1);
                esp_idf_sys::gpio_set_level(sclk_pin, 1);
                esp_idf_sys::esp_rom_delay_us(1);
            }
        }

        // Reset GPIO so SPI driver can take control of the pins
        unsafe {
            esp_idf_sys::gpio_reset_pin(cs_pin);
            esp_idf_sys::gpio_reset_pin(sclk_pin);
            esp_idf_sys::gpio_reset_pin(mosi_pin);
        }

        // Now create SPI device normally (same as original working code)
        let spi_config = SpiConfig::new()
            .baudrate(400.kHz().into())
            .data_mode(esp_idf_hal::spi::config::MODE_0)
            .queue_size(1);
        let device_config = DriverConfig::new().dma(esp_idf_hal::spi::Dma::Auto(4096));

        log::info!("sd: SpiDeviceDriver::new_single …");
        let spi = SpiDeviceDriver::new_single(
            spi,
            sclk,
            mosi,
            Some(miso),
            Some(cs),
            &device_config,
            &spi_config,
        )
        .unwrap();
        log::info!("sd: SpiDeviceDriver::new_single — ok");

        log::info!("sd: SdCard::new …");
        let sdcard = SdCard::new(spi, delay);

        log::info!("sd: probing card (num_bytes) …");
        let card_bytes = sdcard.num_bytes().unwrap();
        log::info!("sd: card size is {} bytes", card_bytes);

        log::info!("sd: VolumeManager::new …");
        let volume_manager = embedded_sdmmc::VolumeManager::new(sdcard, FakeTimesource());
        log::info!("sd: CardputerSd::build — complete");
        CardputerSd { volume_manager }
    }

    pub fn read_file(&mut self, path: &str) -> Result<String, Error<SdCardError>> {
        let volume0 = self.volume_manager.open_volume(VolumeIdx(0))?;
        let root_dir = volume0.open_root_dir()?;

        let file = root_dir.open_file_in_dir(path, Mode::ReadOnly)?;
        let mut contents = Vec::new();

        let mut buffer = [0u8; 512];
        loop {
            let bytes_read = file.read(&mut buffer)?;
            if bytes_read == 0 {
                break;
            }
            contents.extend_from_slice(&buffer[..bytes_read]);
        }

        String::from_utf8(contents).map_err(|_| {
            embedded_sdmmc::Error::FormatError("Failed to convert bytes to String".into())
        })
    }

    pub fn write_file(&mut self, path: &str, contents: &str) -> Result<(), Error<SdCardError>> {
        let volume0 = self.volume_manager.open_volume(VolumeIdx(0))?;
        let root_dir = volume0.open_root_dir()?;

        let file = root_dir.open_file_in_dir(path, Mode::ReadWriteCreateOrTruncate)?;
        file.write(contents.as_bytes())?;
        file.flush()?;
        file.close()?;
        Ok(())
    }

    pub fn is_file_exists(&mut self, path: &str) -> Result<bool, Error<SdCardError>> {
        let volume0 = self.volume_manager.open_volume(VolumeIdx(0))?;
        let root_dir = volume0.open_root_dir()?;
        let file = root_dir.open_file_in_dir(path, Mode::ReadOnly);
        Ok(file.is_ok())
    }
}
