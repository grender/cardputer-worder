use embedded_hal::delay::DelayNs;
use embedded_sdmmc::{
    BlockDevice, Directory, Error, Mode, SdCard, TimeSource, VolumeIdx, VolumeManager,
};
use embedded_sdmmc::sdcard::AcquireOpts;
use esp_idf_hal::{
    delay::Delay,
    gpio::{InputPin, OutputPin},
    spi::{
        config::DriverConfig, SpiAnyPins, SpiConfig, SpiDeviceDriver, SpiDriver,
    },
    units::FromValueType,
};

use embedded_sdmmc::SdCardError;
use esp_idf_sys;

pub struct CardputerSd<'a, DELAYER>
where
    DELAYER: DelayNs + 'a,
{
    // sdcard: SdCard<SpiDeviceDriver<'a, SpiDriver<'a>>, DELAYER>,
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
        log::info!("sd: CardputerSd::build — start (SPI3, DMA off)");
        let delay = Delay::new_default();

        // SD spec requires ≤400kHz during initialization.
        // Using 400kHz avoids flaky cold-start behavior on some cards.
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

        // Give the SD card time to power up before first command
        delay.delay_ms(100);

        log::info!("sd: SdCard::new (acquire_retries=10) …");
        let opts = AcquireOpts {
            acquire_retries: 10,
            ..AcquireOpts::default()
        };
        let sdcard = SdCard::new_with_options(spi, delay, opts);

        log::info!("sd: probing card (num_bytes), retrying up to 5 times …");
        // Remove main task from watchdog during SD init (can take seconds on cold start)
        unsafe { esp_idf_sys::esp_task_wdt_delete(esp_idf_sys::xTaskGetCurrentTaskHandle()); }
        let mut card_bytes = None;
        for attempt in 1..=5 {
            match sdcard.num_bytes() {
                Ok(bytes) => {
                    card_bytes = Some(bytes);
                    break;
                }
                Err(e) => {
                    log::warn!("sd: attempt {} failed: {:?}, waiting 500ms…", attempt, e);
                    sdcard.mark_card_uninit();
                    // Use esp_idf delay since we consumed `delay` into SdCard
                    unsafe { esp_idf_sys::vTaskDelay(50); } // ~500ms (tick = 10ms)
                }
            }
        }
        unsafe { esp_idf_sys::esp_task_wdt_add(esp_idf_sys::xTaskGetCurrentTaskHandle()); }
        let card_bytes = card_bytes.expect("SD card init failed after 5 attempts");
        log::info!("sd: card size is {} bytes", card_bytes);

        log::info!("sd: VolumeManager::new …");
        let volume_manager = embedded_sdmmc::VolumeManager::new(sdcard, FakeTimesource());
        log::info!("sd: CardputerSd::build — complete");
        return CardputerSd {
            volume_manager: volume_manager,
        };
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

        let file = root_dir.open_file_in_dir(path, Mode::ReadWriteCreate)?;
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

fn list_dir<
    B: BlockDevice,
    T: TimeSource,
    const MAX_DIRS: usize,
    const MAX_FILES: usize,
    const MAX_VOLUMES: usize,
>(
    directory: Directory<B, T, MAX_DIRS, MAX_FILES, MAX_VOLUMES>,
    path: &str,
) -> Result<(), embedded_sdmmc::Error<B::Error>> {
    log::info!("Listing {}", path);
    let mut children = Vec::new();
    directory.iterate_dir(|entry| {
        log::info!(
            "{:12} {:9} {} {}",
            entry.name,
            entry.size,
            entry.mtime,
            if entry.attributes.is_directory() {
                "<DIR>"
            } else {
                ""
            }
        );
        if entry.attributes.is_directory()
            && entry.name != embedded_sdmmc::ShortFileName::parent_dir()
            && entry.name != embedded_sdmmc::ShortFileName::this_dir()
        {
            children.push(entry.name.clone());
        }
    })?;
    for child_name in children {
        let child_dir = directory.open_dir(&child_name)?;
        let child_path = if path == "/" {
            format!("/{}", child_name)
        } else {
            format!("{}/{}", path, child_name)
        };
        list_dir(child_dir, &child_path)?;
    }
    Ok(())
}
