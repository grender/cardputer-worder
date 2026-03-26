use core::convert::Infallible;

use display_interface::{DataFormat, DisplayError};
use embedded_graphics::{
    pixelcolor::Rgb565,
    prelude::{IntoStorage, Point},
};
use embedded_graphics_framebuf::FrameBuf;
use esp_idf_hal::gpio::OutputPin;
use esp_idf_hal::spi::SpiAnyPins;
use esp_idf_sys;
use mipidsi::dcs::{SetColumnAddress, SetPageAddress, WriteMemoryStart};
use esp_idf_hal::delay::FreeRtos;

use super::{
    display::{CardputerDisplay, DISPLAY_SIZE_HEIGHT, DISPLAY_SIZE_WIDTH},
    framebuffer::CardputerFramebuffer,
};
use display_interface::WriteOnlyDataCommand;

pub struct CardputerScreen<'a> {
    cardputer_display: CardputerDisplay<'a>,
    pub framebuffer: FrameBuf<Rgb565, CardputerFramebuffer>,
    flush_sample_count: u64,
    flush_total_us_acc: u64,
    flush_cmd_us_acc: u64,
    flush_iter_us_acc: u64,
    flush_delay_us_acc: u64,
    flush_send_us_acc: u64,
}

impl<'a> embedded_graphics::geometry::OriginDimensions for CardputerScreen<'a> {
    fn size(&self) -> embedded_graphics::prelude::Size {
        self.framebuffer.size()
    }
}

impl<'a> embedded_graphics::draw_target::DrawTarget for CardputerScreen<'a> {
    type Color = Rgb565;

    type Error = Infallible;

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = embedded_graphics::Pixel<Self::Color>>,
    {
        self.framebuffer.draw_iter(pixels)
    }

    fn fill_contiguous<I>(
        &mut self,
        area: &embedded_graphics::primitives::Rectangle,
        colors: I,
    ) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Self::Color>,
    {
        self.framebuffer.fill_contiguous(area, colors)
    }

    fn fill_solid(
        &mut self,
        area: &embedded_graphics::primitives::Rectangle,
        color: Self::Color,
    ) -> Result<(), Self::Error> {
        self.framebuffer.fill_solid(area, color)
    }

    fn clear(&mut self, color: Self::Color) -> Result<(), Self::Error> {
        self.framebuffer.clear(color)
    }
}

impl<'a> CardputerScreen<'a> {
    pub fn into_parts(
        self,
    ) -> (
        super::display::CardputerDisplay<'a>,
        FrameBuf<Rgb565, CardputerFramebuffer>,
    ) {
        (self.cardputer_display, self.framebuffer)
    }

    pub fn build<SPI>(
        initial_color: Rgb565,
        spi: SPI,
        sck: impl OutputPin + 'a,
        dc: impl OutputPin + 'a,
        cs: impl OutputPin + 'a,
        rs: impl OutputPin + 'a,
        rst: impl OutputPin + 'a,
        bl: impl OutputPin + 'a,
    ) -> CardputerScreen<'a>
    where
        SPI: SpiAnyPins + 'a,
    {
        log::info!("CardputerScreen::build — calling display::build");
        let display = super::display::build(spi, sck, dc, cs, rs, rst, bl).unwrap();
        log::info!("CardputerScreen::build — framebuffer …");
        let framebuffer_data = CardputerFramebuffer::new(initial_color);
        let framebuffer = FrameBuf::new_with_origin(framebuffer_data, 240, 135, Point::new(52, 40));
        log::info!("CardputerScreen::build — done");
        CardputerScreen {
            cardputer_display: display,
            framebuffer: framebuffer,
            flush_sample_count: 0,
            flush_total_us_acc: 0,
            flush_cmd_us_acc: 0,
            flush_iter_us_acc: 0,
            flush_delay_us_acc: 0,
            flush_send_us_acc: 0,
        }
    }

    pub fn backlight_off(&mut self) -> Result<(), DisplayError> {
        self.cardputer_display.backlight_pin.set_low().map_err(|_| DisplayError::BusWriteError)
    }

    pub fn backlight_on(&mut self) -> Result<(), DisplayError> {
        self.cardputer_display.backlight_pin.set_high().map_err(|_| DisplayError::BusWriteError)
    }

    pub fn flush_framebuffer(&mut self) -> Result<(), DisplayError> {
        //let mut screen: mipidsi::Display<display_interface_spi::SPIInterface<esp_idf_hal::spi::SpiDeviceDriver<'_, esp_idf_hal::spi::SpiDriver<'_>>, esp_idf_hal::gpio::PinDriver<'_, Gpio34, esp_idf_hal::gpio::Output>>, super::st7789v2::ST7789V2, esp_idf_hal::gpio::PinDriver<'_, Gpio33, esp_idf_hal::gpio::Output>> = self.cardputer_display.screen;
        let t0 = unsafe { esp_idf_sys::esp_timer_get_time() as u64 };
        let screen = &mut self.cardputer_display.screen;
        unsafe {
            screen.dcs().write_command(SetColumnAddress::new(40, 279))?;

            screen.dcs().write_command(SetPageAddress::new(53, 187))?;

            screen.dcs().write_command(WriteMemoryStart)?;
            let t_after_cmd = esp_idf_sys::esp_timer_get_time() as u64;

            //let buf = DataFormat::U8(framebuffer_data);
            let pixel_data: &[u16] = &self.framebuffer.data.data;
            screen.dcs().di.send_data(DataFormat::U16(pixel_data))?;

            let t_after_iter = esp_idf_sys::esp_timer_get_time() as u64;
            let t_after_delay = esp_idf_sys::esp_timer_get_time() as u64;
            let t_after_send = esp_idf_sys::esp_timer_get_time() as u64;

            let cmd_us = t_after_cmd - t0;
            let iter_us = t_after_iter - t_after_cmd;
            let delay_us = t_after_delay - t_after_iter;
            let send_us = t_after_send - t_after_delay;
            let total_us = t_after_send - t0;

            self.flush_sample_count = self.flush_sample_count.wrapping_add(1);
            self.flush_total_us_acc += total_us;
            self.flush_cmd_us_acc += cmd_us;
            self.flush_iter_us_acc += iter_us;
            self.flush_delay_us_acc += delay_us;
            self.flush_send_us_acc += send_us;

            if self.flush_sample_count >= 30 || total_us > 1_000_000 {
                let n = self.flush_sample_count.max(1);
                log::info!(
                    "perf flush_framebuffer avg us (total={}, cmd={}, iter={}, delay={}, send={}); last us (total={}, cmd={}, iter={}, delay={}, send={})",
                    self.flush_total_us_acc / n,
                    self.flush_cmd_us_acc / n,
                    self.flush_iter_us_acc / n,
                    self.flush_delay_us_acc / n,
                    self.flush_send_us_acc / n,
                    total_us,
                    cmd_us,
                    iter_us,
                    delay_us,
                    send_us
                );
                self.flush_sample_count = 0;
                self.flush_total_us_acc = 0;
                self.flush_cmd_us_acc = 0;
                self.flush_iter_us_acc = 0;
                self.flush_delay_us_acc = 0;
                self.flush_send_us_acc = 0;
            }
        }
        Ok(())
    }
}
