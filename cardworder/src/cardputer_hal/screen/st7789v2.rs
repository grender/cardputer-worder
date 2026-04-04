use embedded_graphics::pixelcolor::Rgb565;
use embedded_hal::delay::DelayNs;

use mipidsi::{
    dcs::{
        BitsPerPixel, EnterNormalMode, ExitSleepMode, InterfaceExt, PixelFormat,
        SetAddressMode, SetDisplayOn, SetInvertMode, SetPixelFormat, SetScrollArea, SoftReset,
    },
    interface::Interface,
    models::{Model, ModelInitError},
    options::{ColorInversion, ModelOptions},
};

#[derive(Clone, Copy)]
pub struct ST7789V2;

impl Model for ST7789V2 {
    type ColorFormat = Rgb565;
    const FRAMEBUFFER_SIZE: (u16, u16) = (240, 320);

    fn init<DELAY, DI>(
        &mut self,
        di: &mut DI,
        delay: &mut DELAY,
        options: &ModelOptions,
    ) -> Result<SetAddressMode, ModelInitError<DI::Error>>
    where
        DELAY: DelayNs,
        DI: Interface,
    {
        log::info!("ST7789V2 init");
        let madctl = SetAddressMode::from(options);

        di.write_command(SoftReset).map_err(ModelInitError::Interface)?;
        delay.delay_us(150_000);

        di.write_command(ExitSleepMode).map_err(ModelInitError::Interface)?;
        delay.delay_us(10_000);

        di.write_command(SetInvertMode::new(ColorInversion::Normal))
            .map_err(ModelInitError::Interface)?;

        di.write_command(SetScrollArea::new(0, Self::FRAMEBUFFER_SIZE.1, 0))
            .map_err(ModelInitError::Interface)?;

        di.write_command(madctl).map_err(ModelInitError::Interface)?;

        let pf = PixelFormat::with_all(BitsPerPixel::from_rgb_color::<Self::ColorFormat>());
        di.write_command(SetPixelFormat::new(pf))
            .map_err(ModelInitError::Interface)?;

        di.write_command(SetInvertMode::new(ColorInversion::Inverted))
            .map_err(ModelInitError::Interface)?;
        delay.delay_us(10_000);

        di.write_command(SetInvertMode::new(options.invert_colors))
            .map_err(ModelInitError::Interface)?;
        delay.delay_us(10_000);

        di.write_command(EnterNormalMode).map_err(ModelInitError::Interface)?;
        delay.delay_us(10_000);

        di.write_command(SetDisplayOn).map_err(ModelInitError::Interface)?;
        delay.delay_us(10_000);

        Ok(madctl)
    }
}
