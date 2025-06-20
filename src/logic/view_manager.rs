use embedded_graphics::{pixelcolor::Rgb565, prelude::RgbColor};

use crate::{cardputer_hal::cardputer_hal::{CardputerHal, KeyboardState}, ui::cardworder_ui::CardworderUi};

pub struct ViewManager<'a> {
    hal: CardputerHal<'a>,
    ui: CardworderUi<'a>,
    current_view: Box<dyn CardputerView>,
    view_need_init: bool,
}

pub trait CardputerView {
    fn is_need_clear_on_update(&self) -> bool;
    fn is_need_top_line(&self) -> bool;

    fn init(&mut self, hal: &mut CardputerHal<'_>, ui: &mut CardworderUi<'_>);
    fn update(&mut self, keyboard_state: &KeyboardState) -> Option<Box<dyn CardputerView>>;
    fn draw(&mut self, ui: &mut CardworderUi<'_>);
}

impl <'a> ViewManager<'a> {
    pub fn new(hal: CardputerHal<'a>, ui: CardworderUi<'a>, view: Box<dyn CardputerView>) -> Self {
        Self { hal, ui, current_view: view, view_need_init: true }
    }

    pub fn loop_logic(&mut self) {
        self.hal.update_keyboard_state();

        if self.view_need_init {
            self.current_view.init(&mut self.hal, &mut self.ui);
            self.view_need_init = false;
        }

        let next_view = self.current_view.update(&self.hal.keyboard_state);
        if let Some(next_view) = next_view {
            self.current_view = next_view;
            self.view_need_init = true;
            self.ui.clear(Rgb565::BLACK);
            self.current_view.update(&self.hal.keyboard_state);
        }

        if self.current_view.is_need_clear_on_update() {
            self.ui.clear(Rgb565::BLACK);
        }

        self.current_view.draw(&mut self.ui);
        if self.current_view.is_need_top_line() {
            self.ui.draw_top_line(&self.hal.keyboard_state.input_state, &self.hal.keyboard_state.pressed);
        }
        self.ui.flip_buffer();
    }
}