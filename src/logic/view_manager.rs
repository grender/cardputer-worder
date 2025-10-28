use embedded_graphics::{pixelcolor::Rgb565, prelude::RgbColor};

use crate::{cardputer_hal::cardputer_hal::{CardputerHal, KeyboardState}, ui::cardworder_ui::CardworderUi};

pub struct ViewManager<'a> {
    hal: CardputerHal<'a>,
    ui: CardworderUi<'a>,
    current_view: Box<dyn CardputerView<'a>>,
    view_need_init: bool,
}

pub trait CardputerView<'a> {
    fn is_need_clear_on_update(&'a self) -> bool;
    fn is_need_top_line(&'a self) -> bool;

    fn init(&'a mut self, hal: &mut CardputerHal<'_>, ui: &mut CardworderUi<'_>);
    fn destruct(&'a mut self);
    fn update(&'a mut self, keyboard_state: &KeyboardState) -> Option<Box<dyn CardputerView<'a>>>;
    fn draw(&'a self, ui: &mut CardworderUi<'_>);

    fn form(&'a self) -> Vec<crate::logic::views::UiLineType<'a>>;
}

impl <'a> ViewManager<'a> {
    pub fn new(hal: CardputerHal<'a>, ui: CardworderUi<'a>, view: Box<dyn CardputerView<'a>>) -> Self {
        Self { hal, ui, current_view: view, view_need_init: true }
    }

    pub fn loop_logic(&'a mut self) {
        self.hal.update_keyboard_state();

        if self.view_need_init {
            self.current_view.init(&mut self.hal, &mut self.ui);
            self.view_need_init = false;
        }

        let next_view = self.current_view.update(&self.hal.keyboard_state);
        if let Some(next_view) = next_view {
            self.current_view.destruct();
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