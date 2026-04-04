use core::convert::Infallible;

use embedded_graphics::{
    pixelcolor::Rgb565,
    prelude::Point,
};
use embedded_graphics_framebuf::FrameBuf;
use esp_idf_hal::gpio::OutputPin;
use esp_idf_hal::spi::SpiAnyPins;
use crate::esp_util;

use super::{
    display::{CardputerDisplay, DisplayError},
    framebuffer::CardputerFramebuffer,
};

pub struct CardputerScreen<'a> {
    cardputer_display: CardputerDisplay<'a>,
    pub framebuffer: FrameBuf<Rgb565, CardputerFramebuffer>,
    flush_sample_count: u64,
    flush_total_us_acc: u64,
    flush_cmd_us_acc: u64,
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
            flush_send_us_acc: 0,
        }
    }

    pub fn backlight_off(&mut self) -> Result<(), DisplayError> {
        self.cardputer_display.backlight_pin.set_low().map_err(|_| DisplayError::Interface)
    }

    pub fn backlight_on(&mut self) -> Result<(), DisplayError> {
        self.cardputer_display.backlight_pin.set_high().map_err(|_| DisplayError::Interface)
    }

    pub fn flush_framebuffer(&mut self) -> Result<(), DisplayError> {
        let t0 = esp_util::now_us();

        let t_after_cmd = esp_util::now_us();

        self.cardputer_display.write_pixels(40, 279, 53, 187, &self.framebuffer.data.data);

        {
            let t_after_send = esp_util::now_us();
            let total_us = t_after_send - t0;
            let cmd_us = t_after_cmd - t0;
            let send_us = t_after_send - t_after_cmd;

            self.flush_sample_count = self.flush_sample_count.wrapping_add(1);
            self.flush_total_us_acc += total_us;
            self.flush_cmd_us_acc += cmd_us;
            self.flush_send_us_acc += send_us;

            if self.flush_sample_count >= 30 || total_us > 1_000_000 {
                let n = self.flush_sample_count.max(1);
                log::info!(
                    "perf flush_framebuffer avg us (total={}, cmd={}, send={})",
                    self.flush_total_us_acc / n, self.flush_cmd_us_acc / n, self.flush_send_us_acc / n,
                );
                self.flush_sample_count = 0;
                self.flush_total_us_acc = 0;
                self.flush_cmd_us_acc = 0;
                self.flush_send_us_acc = 0;
            }
        }
        Ok(())
    }
}
