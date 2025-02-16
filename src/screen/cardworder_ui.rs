use display_interface::DisplayError;
use embedded_fps::{StdClock, FPS};
use embedded_graphics::geometry::AnchorPoint;
use embedded_graphics::mono_font::ascii::FONT_9X18_BOLD;
use embedded_graphics::prelude::WebColors;
use embedded_graphics::primitives::Line;
use embedded_graphics::text::{Alignment, Baseline, TextStyleBuilder};
use embedded_graphics::{
    mono_font::{ascii::FONT_4X6, MonoTextStyle},
    prelude::{Point, RgbColor},
    text::Text,
};
use esp_idf_svc::hal::prelude::Peripherals;

use embedded_graphics::{pixelcolor::Rgb565, prelude::*, primitives::PrimitiveStyle};

use u8g2_fonts::types::{FontColor, HorizontalAlignment, VerticalPosition};
use u8g2_fonts::{fonts, FontRenderer};

use crate::screen::cardputer_screen::CardputerScreen;

pub struct CardworderUi<'a> {
    screen: CardputerScreen<'a>,
    fps_counter: FPS<45, StdClock>,
    debug_small_text_style: MonoTextStyle<'a, Rgb565>,
}

impl CardworderUi<'_> {
    pub fn build<'b>(screen: CardputerScreen<'b>) -> CardworderUi<'b> {
        let fps_counter =  FPS::<45, _>::new(StdClock::default());
        let debug_small_text_style = MonoTextStyle::new(&FONT_4X6, Rgb565::WHITE);
        CardworderUi {
            screen: screen,
            fps_counter: fps_counter,
            debug_small_text_style,
        }
    }

    pub fn clear(&mut self, color:Rgb565) {
        self.screen.clear(color);
    }

    pub fn flip_buffer(&mut self) {
        let fps = self.fps_counter.tick();
        let fps_text = format!("FPS: {}", fps);
        let text_style = TextStyleBuilder::new().baseline(Baseline::Top).alignment(Alignment::Left).build();
        let text = Text::with_text_style(
            &fps_text,
            Point::new(0, 0),
            self.debug_small_text_style,
            text_style
        );

        let text_box = text.bounding_box();
        
        let mut new_size = text_box.size.clone();
        new_size.height +=1;
        new_size.width +=1;
        let fill_box = text_box.resized(new_size, AnchorPoint::TopLeft);        
        self.screen.fill_solid(&fill_box, Rgb565::CSS_BLACK).unwrap()
        ;

        text
        .draw(&mut self.screen)
        .unwrap();
        //.unwrap_or_log("error draw fps text")

        self.screen.flush_framebuffer();
    }
}
