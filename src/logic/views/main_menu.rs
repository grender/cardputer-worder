use embedded_graphics::{pixelcolor::Rgb565, prelude::RgbColor};

use crate::{cardputer_hal::{cardputer_hal::{CardputerHal, KeyboardState}, input::keyboard_io::{KeyEvent, Scancode}}, logic::view_manager::CardputerView, ui::cardworder_ui::CardworderUi};


pub struct MainMenuView {
    show_fps: bool,
}

impl Default for MainMenuView {
    fn default() -> Self {
        Self { show_fps: false }
    }
}

impl CardputerView for MainMenuView {
    fn is_need_top_line(&self) -> bool {
        true
    }

    fn init(&mut self, hal: &mut CardputerHal<'_>, ui: &mut CardworderUi<'_>) {
    }

    fn update(&mut self, keyboard_state: &KeyboardState) -> Option<Box<dyn CardputerView>> {

        match (keyboard_state.input_state.opt_pressed, keyboard_state.key) {
            (true, Some((KeyEvent::Pressed, Scancode::F))) => {
                self.show_fps = !self.show_fps;
            }
            _ => {}
        }

        None
    }

    fn draw(&mut self, ui: &mut CardworderUi<'_>) {
        ui.draw_text_huge("Hello world!", 0, 8, Rgb565::WHITE);
        ui.show_fps = self.show_fps;
    }
}