//! Create and initialize ST7789 display driver
use anyhow::Result;
use esp_idf_hal::{
    delay::Delay,
    gpio::{AnyInputPin, Output, OutputPin, PinDriver},
    spi::{config::DriverConfig, SpiAnyPins, SpiConfig, SpiDeviceDriver, SpiDriver},
    units::FromValueType,
};
use mipidsi::{
    dcs::{InterfaceExt, SetColumnAddress, SetPageAddress, SetScrollStart, WriteMemoryStart},
    interface::{Interface, SpiInterface},
    options::{
        ColorInversion, ColorOrder, HorizontalRefreshOrder, Orientation, RefreshOrder, Rotation,
        VerticalRefreshOrder,
    },
    Builder, Display,
};

use crate::cardputer_hal::screen::st7789v2::ST7789V2;

/// Display error type (replaces display_interface::DisplayError)
#[derive(Debug)]
pub enum DisplayError {
    Interface,
}

type Drawable<'a> = Display<
    SpiInterface<'a, SpiDeviceDriver<'a, SpiDriver<'a>>, PinDriver<'a, Output>>,
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

impl<'a> CardputerDisplay<'a> {
    /// Send DCS commands to write a region of pixel data.
    /// Encapsulates the unsafe dcs() access for the common flush pattern.
    pub fn write_pixels(&mut self, col_start: u16, col_end: u16, page_start: u16, page_end: u16, pixels: &[u16]) {
        // SAFETY: dcs() is unsafe because it bypasses mipidsi's state tracking.
        // We only use it for pixel writes which don't conflict with driver state.
        let di = unsafe { self.screen.dcs() };
        di.write_command(SetColumnAddress::new(col_start, col_end)).unwrap();
        di.write_command(SetPageAddress::new(page_start, page_end)).unwrap();
        di.write_command(WriteMemoryStart).unwrap();
        di.send_pixels(pixels.iter().map(|&p| p.to_ne_bytes())).unwrap();
    }

    /// Set hardware scroll offset (4-byte SPI command, no pixel data).
    pub fn set_scroll_start(&mut self, offset: u16) {
        let di = unsafe { self.screen.dcs() };
        di.write_command(SetScrollStart::new(offset)).unwrap();
    }
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
    log::info!("display: build — start (SPI2, DMA 64K, queue 4)");
    let spi_config = SpiConfig::new()
        .baudrate(80.MHz().into())
        .data_mode(esp_idf_hal::spi::config::MODE_0)
        .polling(false)
        .write_only(true)
        .queue_size(4);
    let device_config = DriverConfig::new().dma(esp_idf_hal::spi::Dma::Auto(32768));

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

    let mut bl = PinDriver::output(bl)?;
    bl.set_low()?;

    // SpiInterface needs a buffer for command/data framing
    let buffer = Box::leak(Box::new([0u8; 512]));

    log::info!("display: mipidsi Builder::init …");
    let mut drawable = Builder::new(model, SpiInterface::new(spi, rs, buffer))
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

    // Clear display RAM before turning backlight on (no random pixels)
    log::info!("display: clearing before backlight …");
    use embedded_graphics::prelude::{DrawTarget, RgbColor};
    use embedded_graphics::pixelcolor::Rgb565;
    drawable.clear(Rgb565::BLACK).ok();

    log::info!("display: backlight pulse");
    bl.set_low()?;
    delay.delay_us(10_000);
    bl.set_high()?;
    delay.delay_us(10_000);

    log::info!("display: init complete");

    Ok(CardputerDisplay {
        screen: drawable,
        backlight_pin: bl,
    })
}
