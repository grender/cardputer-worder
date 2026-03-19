//! Create and initialize ST7789 display driver
use anyhow::Result;
use display_interface_spi::SPIInterface;
use esp_idf_hal::{
    delay::Delay,
    gpio::{AnyInputPin, Output, OutputPin, PinDriver},
    spi::{config::DriverConfig, SpiAnyPins, SpiConfig, SpiDeviceDriver, SpiDriver},
    units::FromValueType,
};
use mipidsi::{
    options::{
        ColorInversion, ColorOrder, HorizontalRefreshOrder, Orientation, RefreshOrder, Rotation,
        VerticalRefreshOrder,
    },
    Builder, Display,
};

use crate::cardputer_hal::screen::st7789v2::ST7789V2;

type Drawable<'a> = Display<
    SPIInterface<SpiDeviceDriver<'a, SpiDriver<'a>>, PinDriver<'a, Output>>,
    ST7789V2,
    PinDriver<'a, Output>,
>;

/// Display width
pub const DISPLAY_SIZE_WIDTH: u16 = 240;
/// Display height
pub const DISPLAY_SIZE_HEIGHT: u16 = 135;

pub struct CardputerDisplay<'a> {
    pub screen: Drawable<'a>,
    pub backlight_pin: PinDriver<'a, Output>,
}

pub fn build<'a, SPI>(
    spi: SPI,
    sck: impl OutputPin + 'a,
    dc: impl OutputPin + 'a,
    cs: impl OutputPin + 'a,
    rs: impl OutputPin + 'a,
    rst: impl OutputPin + 'a,
    bl: impl OutputPin + 'a,
) -> Result<CardputerDisplay<'a>>
where
    SPI: SpiAnyPins + 'a,
{
    log::info!("display: build — start (SPI2, DMA off, queue 1)");
    let spi_config = SpiConfig::new()
        .baudrate(80.MHz().into())
        .data_mode(esp_idf_hal::spi::config::MODE_0)
        .queue_size(4);
    let device_config = DriverConfig::new().dma(esp_idf_hal::spi::Dma::Auto(4096));

    log::info!("display: SpiDeviceDriver::new_single …");
    let spi = SpiDeviceDriver::new_single(
        spi,
        sck,
        dc,
        None::<AnyInputPin<'_>>,
        Some(cs),
        &device_config,
        &spi_config,
    )?;
    log::info!("display: SpiDeviceDriver::new_single — ok");

    let model: ST7789V2 = ST7789V2 {};

    let mut delay = Delay::new_default();

    log::info!("display: RS/RST as output …");
    let rs = PinDriver::output(rs)?;
    let rst = PinDriver::output(rst)?;

    log::info!("display: backlight pulse …");
    let mut bl = PinDriver::output(bl)?;
    bl.set_low()?;
    delay.delay_us(10_000);
    bl.set_high()?;
    delay.delay_us(10_000);

    log::info!("display: mipidsi Builder::init …");
    let drawable = Builder::new(model, SPIInterface::new(spi, rs))
        .reset_pin(rst)
        .display_size(DISPLAY_SIZE_HEIGHT, DISPLAY_SIZE_WIDTH)
        .display_offset(52, 40)
        .invert_colors(ColorInversion::Inverted)
        .orientation(Orientation::new().rotate(Rotation::Deg90))
        .refresh_order(RefreshOrder::new(
            VerticalRefreshOrder::BottomToTop,
            HorizontalRefreshOrder::LeftToRight,
        ))
        .color_order(ColorOrder::Rgb)
        .init(&mut delay)
        .map_err(|e| {
            log::info!("got error 1 {e:?}");
            anyhow::Error::msg("unknown")
        })?;

    log::info!("display: mipidsi init — ok, build complete");

    Ok(CardputerDisplay {
        screen: drawable,
        backlight_pin: bl,
    })
}
