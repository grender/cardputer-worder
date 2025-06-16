use std::iter;

use embedded_graphics::pixelcolor::Rgb565;
use embedded_graphics_framebuf::backends::FrameBufferBackend;

use super::display::{DISPLAY_SIZE_HEIGHT, DISPLAY_SIZE_WIDTH};

const DISPLAY_SIZE_WIDTH_U: usize = DISPLAY_SIZE_WIDTH as usize;
const DISPLAY_SIZE_HEIGHT_U: usize = DISPLAY_SIZE_HEIGHT as usize;

pub struct CardputerFramebuffer {
    pub data: Vec<Rgb565>,
}

impl FrameBufferBackend for CardputerFramebuffer {
    type Color = Rgb565;

    fn set(&mut self, index: usize, color: Self::Color) {
        self.data[index] = color;
    }

    fn get(&self, index: usize) -> Self::Color {
        self.data[index]
    }

    fn nr_elements(&self) -> usize {
        self.data.len()
    }
}

impl CardputerFramebuffer {
    pub fn new(initial_color: Rgb565) -> Self {
        let fb_data = iter::repeat(initial_color)
            .take(DISPLAY_SIZE_WIDTH_U * DISPLAY_SIZE_HEIGHT_U)
            .collect();
        CardputerFramebuffer { data: fb_data }
    }
}
