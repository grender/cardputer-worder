use std::convert::Infallible;

use display_interface::{DataFormat, DisplayError};
use embedded_graphics::{
    pixelcolor::Rgb565,
    prelude::{DrawTarget, IntoStorage, Point},
};
use embedded_graphics_framebuf::FrameBuf;
use esp_idf_hal::{gpio::{Gpio33, Gpio34, Gpio35, Gpio36, Gpio37, Gpio38}, peripheral::Peripheral, spi::SpiAnyPins};
use mipidsi::dcs::{SetColumnAddress, SetPageAddress, WriteMemoryStart};

use super::{display::CardputerDisplay, framebuffer::CardputerFramebuffer};
use display_interface::WriteOnlyDataCommand;

pub struct CardputerScreen<'a> {
    cardputer_display: CardputerDisplay<'a>,
    pub framebuffer: FrameBuf<Rgb565, CardputerFramebuffer>,
}

impl <'a> embedded_graphics::geometry::OriginDimensions for CardputerScreen<'a> {
    fn size(&self) -> embedded_graphics::prelude::Size {
        self.framebuffer.size()
    }
} 

impl <'a> embedded_graphics::draw_target::DrawTarget for CardputerScreen<'a> {
    type Color = Rgb565;

    type Error = Infallible;

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = embedded_graphics::Pixel<Self::Color>> {
        self.framebuffer.draw_iter(pixels)
    }
    
    fn fill_contiguous<I>(&mut self, area: &embedded_graphics::primitives::Rectangle, colors: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Self::Color>,
    {
        self.framebuffer.fill_contiguous(area, colors)
    }
    
    fn fill_solid(&mut self, area: &embedded_graphics::primitives::Rectangle, color: Self::Color) -> Result<(), Self::Error> {
        self.framebuffer.fill_solid(area, color)
    }
    
    fn clear(&mut self, color: Self::Color) -> Result<(), Self::Error> {
        self.framebuffer.clear(color)
    }
}

impl CardputerScreen<'_> {
    pub fn build<'a, SPI: SpiAnyPins>(initial_color: Rgb565,
        spi: impl Peripheral<P = SPI> + 'a,
        sck: impl Peripheral<P = Gpio36> + 'a,
        dc: impl Peripheral<P = Gpio35> + 'a,
        cs: impl Peripheral<P = Gpio37> + 'a,
        rs: impl Peripheral<P = Gpio34> + 'a,
        rst: impl Peripheral<P = Gpio33> + 'a,
        bl: impl Peripheral<P = Gpio38> + 'a,
    
    ) -> CardputerScreen<'a> {
        let display = super::display::build(
            spi,
            sck,
            dc,
            cs,
            rs,
            rst,
            bl
        )
        .unwrap();
        let framebuffer_data = CardputerFramebuffer::new(initial_color);
        let framebuffer = FrameBuf::new_with_origin(framebuffer_data, 240, 135, Point::new(52, 40));
        CardputerScreen {
            cardputer_display: display,
            framebuffer: framebuffer,
        }
    }

    pub fn backlight_off(&mut self) {
        self.cardputer_display.backlight_pin.set_low();
    }

    pub fn backlight_on(&mut self) {
        self.cardputer_display.backlight_pin.set_high();
    }

    pub fn flush_framebuffer(&mut self) -> Result<(), DisplayError> {
        //let mut screen: mipidsi::Display<display_interface_spi::SPIInterface<esp_idf_hal::spi::SpiDeviceDriver<'_, esp_idf_hal::spi::SpiDriver<'_>>, esp_idf_hal::gpio::PinDriver<'_, Gpio34, esp_idf_hal::gpio::Output>>, super::st7789v2::ST7789V2, esp_idf_hal::gpio::PinDriver<'_, Gpio33, esp_idf_hal::gpio::Output>> = self.cardputer_display.screen;
        let screen = &mut self.cardputer_display.screen;
        unsafe {
            screen.dcs().write_command(SetColumnAddress::new(40, 279))?;

            screen.dcs().write_command(SetPageAddress::new(53, 187))?;

            screen.dcs().write_command(WriteMemoryStart)?;

            //let buf = DataFormat::U8(framebuffer_data);
            let mut iter = self
                .framebuffer
                .data
                .data
                .clone()
                .into_iter()
                .map(|c| c.into_storage());
            let buf = DataFormat::U16BEIter(&mut iter);
            screen.dcs().di.send_data(buf)?;
        }
        Ok(())
    }
}
