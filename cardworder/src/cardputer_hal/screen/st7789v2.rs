use display_interface::{DataFormat, WriteOnlyDataCommand};
use embedded_graphics::{pixelcolor::Rgb565, prelude::IntoStorage};
use embedded_hal::{delay::DelayNs, digital::OutputPin};

use mipidsi::{
    dcs::{
        BitsPerPixel, Dcs, EnterNormalMode, ExitSleepMode, PixelFormat, SetAddressMode,
        SetDisplayOn, SetInvertMode, SetPixelFormat, SetScrollArea, SoftReset, WriteMemoryStart,
    },
    error::{Error, InitError},
    models::Model,
    options::{ColorInversion, ModelOptions},
};

/// ST7789 display in Rgb565 color mode.
///
/// Interfaces implemented by the [display-interface](https://crates.io/crates/display-interface) are supported.

#[derive(Clone, Copy)]
pub struct ST7789V2;

impl Model for ST7789V2 {
    type ColorFormat = Rgb565;
    const FRAMEBUFFER_SIZE: (u16, u16) = (240, 320);

    fn init<RST, DELAY, DI>(
        &mut self,
        dcs: &mut Dcs<DI>,
        delay: &mut DELAY,
        options: &ModelOptions,
        rst: &mut Option<RST>,
    ) -> Result<SetAddressMode, InitError<RST::Error>>
    where
        RST: OutputPin,
        DELAY: DelayNs,
        DI: WriteOnlyDataCommand,
    {
        log::info!("init");
        let madctl = SetAddressMode::from(options);

        match rst {
            Some(ref mut rst) => self.hard_reset(rst, delay)?,
            None => {}
        }

        dcs.write_command(SoftReset)?;
        delay.delay_us(150_000);
        dcs.write_command(ExitSleepMode)?;
        delay.delay_us(10_000);

        dcs.write_command(SetInvertMode::new(ColorInversion::Normal))?;
        dcs.write_command(SetScrollArea::new(0, Self::FRAMEBUFFER_SIZE.1, 0))?;
        dcs.write_command(madctl)?;

        let pf = PixelFormat::with_all(BitsPerPixel::from_rgb_color::<Self::ColorFormat>());
        dcs.write_command(SetPixelFormat::new(pf))?;
        dcs.write_command(SetInvertMode::new(ColorInversion::Inverted))?;
        delay.delay_us(10_000);
        dcs.write_command(SetInvertMode::new(options.invert_colors))?;
        delay.delay_us(10_000);
        dcs.write_command(EnterNormalMode)?;
        delay.delay_us(10_000);
        dcs.write_command(SetDisplayOn)?;
        delay.delay_us(10_000);
        Ok(madctl)
    }

    fn write_pixels<DI, I>(&mut self, dcs: &mut Dcs<DI>, colors: I) -> Result<(), Error>
    where
        DI: WriteOnlyDataCommand,
        I: IntoIterator<Item = Self::ColorFormat>,
    {
        dcs.write_command(WriteMemoryStart)?;
        let mut iter = colors.into_iter().map(|c| c.into_storage());
        let buf = DataFormat::U16BEIter(&mut iter);
        dcs.di.send_data(buf)?;
        Ok(())
    }

    /// Resets the display using a reset pin.
    fn hard_reset<RST, DELAY>(
        &mut self,
        rst: &mut RST,
        delay: &mut DELAY,
    ) -> Result<(), InitError<RST::Error>>
    where
        RST: OutputPin,
        DELAY: DelayNs,
    {
        log::info!("hard reset");
        rst.set_high().map_err(InitError::Pin)?;
        delay.delay_us(10);
        rst.set_low().map_err(InitError::Pin)?;
        delay.delay_us(10);
        rst.set_high().map_err(InitError::Pin)?;
        delay.delay_us(10);

        Ok(())
    }
}
