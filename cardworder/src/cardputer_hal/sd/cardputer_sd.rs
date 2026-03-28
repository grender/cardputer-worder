use embedded_hal::delay::DelayNs;
use embedded_sdmmc::{
    BlockDevice, Error, Mode, SdCard, VolumeIdx, VolumeManager,
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
        let delay = Delay::new_default();

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

    pub fn read_file_bytes(&mut self, path: &str) -> Result<Vec<u8>, Error<SdCardError>> {
        let volume0 = self.volume_manager.open_volume(VolumeIdx(0))?;
        let root_dir = volume0.open_root_dir()?;
        let file = root_dir.open_file_in_dir(path, Mode::ReadOnly)?;
        let mut contents = Vec::new();
        let mut buffer = [0u8; 512];
        loop {
            let bytes_read = file.read(&mut buffer)?;
            if bytes_read == 0 { break; }
            contents.extend_from_slice(&buffer[..bytes_read]);
        }
        Ok(contents)
    }

    pub fn write_file_bytes(&mut self, path: &str, data: &[u8]) -> Result<(), Error<SdCardError>> {
        let volume0 = self.volume_manager.open_volume(VolumeIdx(0))?;
        let root_dir = volume0.open_root_dir()?;
        let file = root_dir.open_file_in_dir(path, Mode::ReadWriteCreateOrTruncate)?;
        file.write(data)?;
        file.flush()?;
        file.close()?;
        Ok(())
    }

    /// Read `buf.len()` bytes from file at given byte offset. Returns bytes actually read.
    pub fn read_at(&mut self, path: &str, offset: u32, buf: &mut [u8]) -> Result<usize, Error<SdCardError>> {
        let volume0 = self.volume_manager.open_volume(VolumeIdx(0))?;
        let root_dir = volume0.open_root_dir()?;
        let file = root_dir.open_file_in_dir(path, Mode::ReadOnly)?;
        file.seek_from_start(offset)?;
        let n = file.read(buf)?;
        Ok(n)
    }

    /// Write `data` at given byte offset (file must exist, opened read-write).
    pub fn write_at(&mut self, path: &str, offset: u32, data: &[u8]) -> Result<(), Error<SdCardError>> {
        let volume0 = self.volume_manager.open_volume(VolumeIdx(0))?;
        let root_dir = volume0.open_root_dir()?;
        let file = root_dir.open_file_in_dir(path, Mode::ReadWriteCreateOrAppend)?;
        file.seek_from_start(offset)?;
        file.write(data)?;
        file.flush()?;
        file.close()?;
        Ok(())
    }

    /// Append `data` to end of file (create if not exists). Returns the byte offset where data was written.
    pub fn append(&mut self, path: &str, data: &[u8]) -> Result<u32, Error<SdCardError>> {
        let volume0 = self.volume_manager.open_volume(VolumeIdx(0))?;
        let root_dir = volume0.open_root_dir()?;
        let file = root_dir.open_file_in_dir(path, Mode::ReadWriteCreateOrAppend)?;
        let offset = file.length();
        file.seek_from_start(offset)?;
        file.write(data)?;
        file.flush()?;
        file.close()?;
        Ok(offset)
    }

    /// Get file length in bytes. Returns 0 if file doesn't exist.
    pub fn file_length(&mut self, path: &str) -> Result<u32, Error<SdCardError>> {
        let exists = self.is_file_exists(path)?;
        if !exists {
            return Ok(0);
        }
        let volume0 = self.volume_manager.open_volume(VolumeIdx(0))?;
        let root_dir = volume0.open_root_dir()?;
        let file = root_dir.open_file_in_dir(path, Mode::ReadOnly)?;
        let len = file.length();
        file.close()?;
        Ok(len)
    }

    pub fn is_file_exists(&mut self, path: &str) -> Result<bool, Error<SdCardError>> {
        let volume0 = self.volume_manager.open_volume(VolumeIdx(0))?;
        let root_dir = volume0.open_root_dir()?;
        let file = root_dir.open_file_in_dir(path, Mode::ReadOnly);
        Ok(file.is_ok())
    }

    /// Write data to one file + read from two files, all in a single volume open.
    /// Used for rate-and-load-next: write FSRS record, read word text, read next FSRS record.
    pub fn write_and_read_multi(
        &mut self,
        write_path: &str, write_offset: u32, write_data: &[u8],
        read1_path: &str, read1_offset: u32, read1_buf: &mut [u8],
        read2_path: &str, read2_offset: u32, read2_buf: &mut [u8],
    ) -> Result<(usize, usize), Error<SdCardError>> {
        let volume0 = self.volume_manager.open_volume(VolumeIdx(0))?;
        let root_dir = volume0.open_root_dir()?;

        // Write
        let wf = root_dir.open_file_in_dir(write_path, Mode::ReadWriteCreateOrAppend)?;
        wf.seek_from_start(write_offset)?;
        wf.write(write_data)?;
        wf.flush()?;
        wf.close()?;

        // Read 1
        let rf1 = root_dir.open_file_in_dir(read1_path, Mode::ReadOnly)?;
        rf1.seek_from_start(read1_offset)?;
        let n1 = rf1.read(read1_buf)?;

        // Read 2
        let rf2 = root_dir.open_file_in_dir(read2_path, Mode::ReadOnly)?;
        rf2.seek_from_start(read2_offset)?;
        let n2 = rf2.read(read2_buf)?;

        Ok((n1, n2))
    }
}
