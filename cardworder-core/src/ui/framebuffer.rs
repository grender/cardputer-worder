use embedded_graphics::pixelcolor::{raw::RawU16, IntoStorage, Rgb565};
use embedded_graphics_framebuf::backends::FrameBufferBackend;

pub const DISPLAY_WIDTH: usize = 240;
pub const DISPLAY_HEIGHT: usize = 135;

/// Supertrait required by `CardworderUi<FB>`.
/// Adds dirty-region tracking and raw pixel slice access on top of `FrameBufferBackend`.
pub trait CardworderFB: FrameBufferBackend<Color = Rgb565> {
    /// Slice of raw pre-byte-swapped u16 pixels from `start..end`.
    fn raw_pixels(&self, start: usize, end: usize) -> &[u16];
    fn clear_no_dirty(&mut self, color: Rgb565);
    fn mark_rows_dirty(&mut self, min_y: usize, max_y: usize);
    fn take_dirty_bbox(&mut self) -> Option<(usize, usize, usize, usize)>;
}

/// Heap-allocated (Vec<u16>) framebuffer for desktop/simulator use.
/// On ESP32 use `CardputerFramebuffer` (DMA-allocated) from the `cardworder` crate.
pub struct CardworderFramebuffer {
    pub data: Vec<u16>,
    dirty_any: bool,
    dirty_min_x: usize,
    dirty_min_y: usize,
    dirty_max_x: usize,
    dirty_max_y: usize,
}

impl FrameBufferBackend for CardworderFramebuffer {
    type Color = Rgb565;

    fn set(&mut self, index: usize, color: Self::Color) {
        self.data[index] = color.into_storage().swap_bytes();
        let x = index % DISPLAY_WIDTH;
        let y = index / DISPLAY_WIDTH;
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
        Rgb565::from(RawU16::new(self.data[index].swap_bytes()))
    }

    fn nr_elements(&self) -> usize {
        self.data.len()
    }
}

impl CardworderFB for CardworderFramebuffer {
    fn raw_pixels(&self, start: usize, end: usize) -> &[u16] {
        &self.data[start..end]
    }

    fn clear_no_dirty(&mut self, color: Rgb565) {
        let fill = color.into_storage().swap_bytes();
        self.data.fill(fill);
    }

    fn mark_rows_dirty(&mut self, min_y: usize, max_y: usize) {
        if !self.dirty_any {
            self.dirty_min_x = 0;
            self.dirty_max_x = DISPLAY_WIDTH - 1;
            self.dirty_min_y = min_y;
            self.dirty_max_y = max_y;
            self.dirty_any = true;
        } else {
            self.dirty_min_x = 0;
            self.dirty_max_x = DISPLAY_WIDTH - 1;
            self.dirty_min_y = self.dirty_min_y.min(min_y);
            self.dirty_max_y = self.dirty_max_y.max(max_y);
        }
    }

    fn take_dirty_bbox(&mut self) -> Option<(usize, usize, usize, usize)> {
        if !self.dirty_any {
            return None;
        }
        let bbox = (self.dirty_min_x, self.dirty_min_y, self.dirty_max_x, self.dirty_max_y);
        self.dirty_any = false;
        self.dirty_min_x = 0;
        self.dirty_min_y = 0;
        self.dirty_max_x = 0;
        self.dirty_max_y = 0;
        Some(bbox)
    }
}

impl CardworderFramebuffer {
    pub fn new(initial_color: Rgb565) -> Self {
        let count = DISPLAY_WIDTH * DISPLAY_HEIGHT;
        let fill = initial_color.into_storage().swap_bytes();
        CardworderFramebuffer {
            data: vec![fill; count],
            dirty_any: true,
            dirty_min_x: 0,
            dirty_min_y: 0,
            dirty_max_x: DISPLAY_WIDTH - 1,
            dirty_max_y: DISPLAY_HEIGHT - 1,
        }
    }
}
