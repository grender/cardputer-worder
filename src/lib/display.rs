//! Create and initialize ST7789 display driver
use anyhow::Result;
use display_interface_spi::SPIInterface;
use esp_idf_hal::{
    delay::Delay,
    gpio::{AnyIOPin, Gpio33, Gpio34, Gpio35, Gpio36, Gpio37, Gpio38, Output, PinDriver},
    peripheral::Peripheral,
    prelude::*,
    spi::{config::DriverConfig, SpiAnyPins, SpiConfig, SpiDeviceDriver, SpiDriver},
};
use mipidsi::{
    options::{
        ColorInversion, ColorOrder, HorizontalRefreshOrder, Orientation, RefreshOrder, Rotation,
        VerticalRefreshOrder,
    },
    Builder, Display,
};

use super::st7789v2::ST7789V2;

type Drawable<'a> = Display<
    SPIInterface<SpiDeviceDriver<'a, SpiDriver<'a>>, PinDriver<'a, Gpio34, Output>>,
    ST7789V2,
    PinDriver<'a, Gpio33, Output>,
>;

/// Display width
pub const DISPLAY_SIZE_WIDTH: u16 = 240;
/// Display height
pub const DISPLAY_SIZE_HEIGHT: u16 = 135;

pub struct CardputerDisplay<'a> {
    pub screen: Drawable<'a>,
    pub backlight_pin: PinDriver<'a, Gpio38, Output>,
}

pub fn build<'a, SPI>(
    spi: impl Peripheral<P = SPI> + 'a,
    sck: impl Peripheral<P = Gpio36> + 'a,
    dc: impl Peripheral<P = Gpio35> + 'a,
    cs: impl Peripheral<P = Gpio37> + 'a,
    rs: impl Peripheral<P = Gpio34> + 'a,
    rst: impl Peripheral<P = Gpio33> + 'a,
    bl: impl Peripheral<P = Gpio38> + 'a,
) -> Result<CardputerDisplay<'a>>
where
    SPI: SpiAnyPins,
{
    log::info!("start building");
    let spi_config = SpiConfig::new().baudrate(80.MHz().into());
    let device_config = DriverConfig::new();

    let spi = SpiDeviceDriver::new_single(
        spi,
        sck,
        dc,
        Option::<AnyIOPin>::None,
        Some(cs),
        &device_config,
        &spi_config,
    )?;

    let model: ST7789V2 = ST7789V2 {};

    let mut delay = Delay::new_default();

    let rs = PinDriver::output(rs)?;
    let rst = PinDriver::output(rst)?;

    log::info!("activate backlight");
    let mut bl = PinDriver::output(bl)?;
    bl.set_low()?;
    delay.delay_us(10_000);
    bl.set_high()?;
    delay.delay_us(10_000);

    log::info!("create drawable");
    let drawable = Builder::new(model, SPIInterface::new(spi, rs))
        .reset_pin(rst)
        .display_size( DISPLAY_SIZE_HEIGHT,DISPLAY_SIZE_WIDTH)
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

    log::info!("create drawable");

    Ok(CardputerDisplay {
        screen: drawable,
        backlight_pin: bl,
    })
}
