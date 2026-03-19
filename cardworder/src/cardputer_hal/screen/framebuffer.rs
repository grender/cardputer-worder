use core::iter;

use embedded_graphics::pixelcolor::{raw::RawU16, IntoStorage, Rgb565};
use embedded_graphics_framebuf::backends::FrameBufferBackend;

use super::display::{DISPLAY_SIZE_HEIGHT, DISPLAY_SIZE_WIDTH};

const DISPLAY_SIZE_WIDTH_U: usize = DISPLAY_SIZE_WIDTH as usize;
const DISPLAY_SIZE_HEIGHT_U: usize = DISPLAY_SIZE_HEIGHT as usize;

pub struct CardputerFramebuffer {
    /// Raw RGB565 storage values (u16). Using u16 lets the SPI flush path send a slice efficiently.
    pub data: Vec<Rgb565>,

    // Dirty region tracking used by `CardputerScreen::flush_framebuffer()`.
    dirty_any: bool,
    dirty_min_x: usize,
    dirty_min_y: usize,
    dirty_max_x: usize,
    dirty_max_y: usize,
}

impl FrameBufferBackend for CardputerFramebuffer {
    type Color = Rgb565;

    fn set(&mut self, index: usize, color: Self::Color) {
        self.data[index] = color;

        // Track modified pixels since the last flush.
        let x = index % DISPLAY_SIZE_WIDTH_U;
        let y = index / DISPLAY_SIZE_WIDTH_U;
        if !self.dirty_any {
            self.dirty_min_x = x;
            self.dirty_max_x = x;
            self.dirty_min_y = y;
            self.dirty_max_y = y;
            self.dirty_any = true;
        } else {
            self.dirty_min_x = self.dirty_min_x.min(x);
            self.dirty_max_x = self.dirty_max_x.max(x);
            self.dirty_min_y = self.dirty_min_y.min(y);
            self.dirty_max_y = self.dirty_max_y.max(y);
        }
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
            // .map(|c| c.into_storage())
            .collect();
        // Force an initial flush of the whole buffer so the display state is correct.
        CardputerFramebuffer {
            data: fb_data,
            dirty_any: true,
            dirty_min_x: 0,
            dirty_min_y: 0,
            dirty_max_x: DISPLAY_SIZE_WIDTH_U - 1,
            dirty_max_y: DISPLAY_SIZE_HEIGHT_U - 1,
        }
    }

    pub fn take_dirty_bbox(&mut self) -> Option<(usize, usize, usize, usize)> {
        if !self.dirty_any {
            return None;
        }

        let bbox = (
            self.dirty_min_x,
            self.dirty_min_y,
            self.dirty_max_x,
            self.dirty_max_y,
        );

        self.dirty_any = false;
        self.dirty_min_x = 0;
        self.dirty_min_y = 0;
        self.dirty_max_x = 0;
        self.dirty_max_y = 0;

        Some(bbox)
    }
}
