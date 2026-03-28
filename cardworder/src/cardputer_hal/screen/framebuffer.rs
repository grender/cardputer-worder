
use embedded_graphics::pixelcolor::{raw::RawU16, IntoStorage, Rgb565};
use embedded_graphics_framebuf::backends::FrameBufferBackend;

use esp_idf_sys::{heap_caps_malloc, MALLOC_CAP_DMA, MALLOC_CAP_INTERNAL};

use super::display::{DISPLAY_SIZE_HEIGHT, DISPLAY_SIZE_WIDTH};

const DISPLAY_SIZE_WIDTH_U: usize = DISPLAY_SIZE_WIDTH as usize;
const DISPLAY_SIZE_HEIGHT_U: usize = DISPLAY_SIZE_HEIGHT as usize;

pub struct CardputerFramebuffer {
    /// Raw RGB565 storage values (u16). Using u16 lets the SPI flush path send a slice efficiently.
    pub data: &'static mut [u16],

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
        self.data[index] = color.into_storage().swap_bytes();

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
        Rgb565::from(RawU16::new(self.data[index].swap_bytes()))
    }

    fn nr_elements(&self) -> usize {
        self.data.len()
    }
}

impl CardputerFramebuffer {
    pub fn new(initial_color: Rgb565) -> Self {
        let count = DISPLAY_SIZE_WIDTH_U * DISPLAY_SIZE_HEIGHT_U;
        let ptr = unsafe {
            heap_caps_malloc(
                count * 2,
                MALLOC_CAP_DMA | MALLOC_CAP_INTERNAL
            ) as *mut u16
        };
        assert!(!ptr.is_null(), "DMA heap allocation failed");
        let data = unsafe { core::slice::from_raw_parts_mut(ptr, count) };
        let fill = initial_color.into_storage().swap_bytes();
        data.fill(fill);

        CardputerFramebuffer {
            data,
            dirty_any: true,
            dirty_min_x: 0,
            dirty_min_y: 0,
            dirty_max_x: DISPLAY_SIZE_WIDTH_U - 1,
            dirty_max_y: DISPLAY_SIZE_HEIGHT_U - 1,
        }
    }

    /// Fill buffer with color without updating dirty tracking.
    /// Used to avoid marking the entire screen dirty on every frame.
    pub fn clear_no_dirty(&mut self, color: Rgb565) {
        let fill = color.into_storage().swap_bytes();
        self.data.fill(fill);
    }

    /// Expand dirty bbox to include given row range (used to flush previously-drawn areas).
    pub fn mark_rows_dirty(&mut self, min_y: usize, max_y: usize) {
        if !self.dirty_any {
            self.dirty_min_x = 0;
            self.dirty_max_x = DISPLAY_SIZE_WIDTH_U - 1;
            self.dirty_min_y = min_y;
            self.dirty_max_y = max_y;
            self.dirty_any = true;
        } else {
            self.dirty_min_x = 0;
            self.dirty_max_x = DISPLAY_SIZE_WIDTH_U - 1;
            self.dirty_min_y = self.dirty_min_y.min(min_y);
            self.dirty_max_y = self.dirty_max_y.max(max_y);
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
