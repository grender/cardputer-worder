use core::mem::MaybeUninit;
use embedded_sdmmc::{BlockDevice, Directory, TimeSource, VolumeIdx};
use esp_idf_hal::{
    delay::Delay,
    prelude::Peripherals,
    spi::{config::DriverConfig, Spi, SpiConfig, SpiDeviceDriver},
    units::FromValueType,
};
use std::fmt::Debug;

// TODO: remove me and get normal date!
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

fn main() {
    esp_idf_svc::sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    let peripherals = Peripherals::take().unwrap();
    let mut delay = Delay::new_default();

    let spi = peripherals.spi3;
    let sclk = peripherals.pins.gpio40;
    let miso = peripherals.pins.gpio39;
    let mosi = peripherals.pins.gpio14;
    let cs = peripherals.pins.gpio12; //. into_push_pull_output();

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
    let sdcard = embedded_sdmmc::sdcard::SdCard::new(spi, delay);

    log::info!("Card size is {} bytes", sdcard.num_bytes().unwrap());

    let mut volume_manager = embedded_sdmmc::VolumeManager::new(sdcard, FakeTimesource());

    let mut volume0 = volume_manager.open_volume(VolumeIdx(0)).unwrap();
    log::info!("Volume 0: {:?}", volume0);

    let root_dir = volume0.open_root_dir().unwrap();
    list_dir(root_dir, "/").unwrap();
    /*
    // volume_manager.iterate_dir(root_dir, func)
    if let Ok(file) = volume_manager.open_file_in_dir(root_dir, "MY_FILE.TXT", Mode::ReadOnly) {
        log::info!("File opened: {:?}", file);
    }
    */
    // setup logger
    // To change the log_level change the env section in .cargo/config.toml
    // or remove it and set ESP_LOGLEVEL manually before running cargo run
    // this requires a clean rebuild because of https://github.com/rust-lang/cargo/issues/10358

    log::info!("Logger is setup");
    log::info!("Hello world!");
    loop {
        log::info!("Loop...");
        delay.delay_ms(500u32);
    }
}
