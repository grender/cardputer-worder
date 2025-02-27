use embedded_hal::delay::DelayNs;
use embedded_sdmmc::{BlockDevice, Directory, SdCard, TimeSource};
use esp_idf_hal::{
    delay::Delay,
    spi::{config::DriverConfig, SpiConfig, SpiDeviceDriver},
    units::FromValueType,
};
use esp_idf_hal::{
    gpio::{Gpio12, Gpio14, Gpio39, Gpio40},
    peripheral::Peripheral,
    spi::{SpiAnyPins, SpiDriver},
};

pub struct CardputerSd<'a, DELAYER: DelayNs> {
    pub sdcard: SdCard<SpiDeviceDriver<'a, SpiDriver<'a>>, DELAYER>,
    // root_dir: embedded_sdmmc::directory::Directory<SpiDeviceDriver, esp_idf_hal::time::EspIdfTime, 16, 16, 1>,
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
    pub fn build<'a, SPI: SpiAnyPins>(
        spi: impl Peripheral<P = SPI> + 'a,
        sclk: impl Peripheral<P = Gpio40> + 'a,
        miso: impl Peripheral<P = Gpio39> + 'a,
        mosi: impl Peripheral<P = Gpio14> + 'a,
        cs: impl Peripheral<P = Gpio12> + 'a,
    ) -> CardputerSd<'a, Delay> {
        let delay = Delay::new_default();

        let spi_config = SpiConfig::new()
            .baudrate(1.MHz().into())
            .data_mode(esp_idf_hal::spi::config::MODE_0)
            .queue_size(1);
        let device_config = DriverConfig::new().dma(esp_idf_hal::spi::Dma::Auto(4096));

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

        log::info!("SPI initialized. Initializing SD-Card...");
        let sdcard = SdCard::new(spi, delay);

        log::info!("Card size is {} bytes", sdcard.num_bytes().unwrap());

        /*
        let mut volume_manager = embedded_sdmmc::VolumeManager::new(sdcard, FakeTimesource());

        let mut volume0 = volume_manager.open_volume(VolumeIdx(0)).unwrap();
        log::info!("Volume 0: {:?}", volume0);
    
        let root_dir = volume0.open_root_dir().unwrap();
        */

        return CardputerSd { sdcard: sdcard };
    }
}
fn list_dir<
    B: BlockDevice,
    T: TimeSource,
    const MAX_DIRS: usize,
    const MAX_FILES: usize,
    const MAX_VOLUMES: usize,
>(
    mut directory: Directory<B, T, MAX_DIRS, MAX_FILES, MAX_VOLUMES>,
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
