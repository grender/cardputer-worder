use cardworder_core::ui::cardworder_ui::CardworderDisplay;
use sdl2::pixels::PixelFormatEnum;
use sdl2::rect::Rect;
use sdl2::render::{Canvas, Texture, TextureCreator};
use sdl2::video::{Window, WindowContext};

pub const DISPLAY_WIDTH: u32 = 240;
pub const DISPLAY_HEIGHT: u32 = 135;
pub const SCALE: u32 = 4;

pub struct SdlDisplay {
    canvas: Canvas<Window>,
    texture: Texture<'static>,
}

impl SdlDisplay {
    pub fn new(canvas: Canvas<Window>, texture_creator: &'static TextureCreator<WindowContext>) -> Self {
        let texture = texture_creator
            .create_texture_streaming(PixelFormatEnum::RGB565, DISPLAY_WIDTH, DISPLAY_HEIGHT)
            .expect("Failed to create SDL texture");

        SdlDisplay { canvas, texture }
    }
}

// Safety: SdlDisplay is only ever used from the main thread (in CardworderUi on the
// main thread). The `Send` bound on `CardworderDisplay` exists so it can be boxed;
// we never actually send it across threads.
unsafe impl Send for SdlDisplay {}

impl CardworderDisplay for SdlDisplay {
    fn flush_rows(&mut self, min_y: usize, max_y: usize, pixels: &[u16]) {
        let width = DISPLAY_WIDTH as usize;
        let row_count = max_y - min_y + 1;
        // `pixels` is pre-sliced to exactly [min_y * width .. (max_y+1) * width]
        let src = &pixels[..row_count * width];

        self.texture
            .with_lock(
                Some(Rect::new(0, min_y as i32, DISPLAY_WIDTH, row_count as u32)),
                |buf, _pitch| {
                    for (i, &p) in src.iter().enumerate() {
                        // Framebuffer stores swap_bytes(rgb565).
                        // p.to_be_bytes() = big-endian bytes of p
                        //                 = LE bytes of p.swap_bytes()
                        //                 = LE bytes of raw rgb565
                        // SDL RGB565 on LE reads those as the correct colour.
                        // Example RED: rgb565=0xF800, stored p=0x00F8,
                        //   to_be_bytes=[0x00,0xF8] → SDL reads LE 0xF800 = RED ✓
                        let bytes = p.to_be_bytes();
                        buf[i * 2]     = bytes[0];
                        buf[i * 2 + 1] = bytes[1];
                    }
                },
            )
            .ok();

        // Always copy the full texture to the canvas back-buffer.
        // The texture accumulates all correct pixel state across frames,
        // so a full copy is safe and prevents back-buffer garbage causing flicker.
        self.canvas.copy(&self.texture, None, None).ok();
        self.canvas.present();
    }

    fn set_scroll_start(&mut self, _offset: u16) {
        // no-op for SDL simulator
    }
}
