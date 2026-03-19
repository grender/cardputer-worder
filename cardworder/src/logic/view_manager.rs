use embedded_graphics::{pixelcolor::Rgb565, prelude::RgbColor};

use crate::{cardputer_hal::cardputer_hal::{CardputerHal, KeyboardState}, ui::cardworder_ui::CardworderUi};

pub struct ViewManager<'a> {
    hal: CardputerHal<'a>,
    ui: CardworderUi<'a>,
    current_view: Box<dyn CardputerView>,
    view_need_init: bool,
    frame_counter: u32,
    keyboard_us_acc: u64,
    draw_us_acc: u64,
    clear_us_acc: u64,
    view_draw_us_acc: u64,
    top_line_us_acc: u64,
    flip_us_acc: u64,
    frame_us_acc: u64,
}

pub trait CardputerView {
    fn is_need_clear_on_update(&self) -> bool;
    fn is_need_top_line(&self) -> bool;

    fn init(&mut self, hal: &mut CardputerHal<'_>, ui: &mut CardworderUi<'_>);
    fn destruct(&mut self);
    fn update(&mut self, keyboard_state: &KeyboardState) -> Option<Box<dyn CardputerView>>;
    fn draw(&mut self, ui: &mut CardworderUi<'_>);

    fn form(&mut self) -> Vec<crate::logic::views::UiLineType>;
}

impl <'a> ViewManager<'a> {
    pub fn new(hal: CardputerHal<'a>, ui: CardworderUi<'a>, view: Box<dyn CardputerView>) -> Self {
        Self {
            hal,
            ui,
            current_view: view,
            view_need_init: true,
            frame_counter: 0,
            keyboard_us_acc: 0,
            draw_us_acc: 0,
            clear_us_acc: 0,
            view_draw_us_acc: 0,
            top_line_us_acc: 0,
            flip_us_acc: 0,
            frame_us_acc: 0,
        }
    }

    pub fn loop_logic(&mut self) {
        let frame_start = unsafe { esp_idf_svc::sys::esp_timer_get_time() as u64 };
        let kb_start = unsafe { esp_idf_svc::sys::esp_timer_get_time() as u64 };
        self.hal.update_keyboard_state();
        let kb_end = unsafe { esp_idf_svc::sys::esp_timer_get_time() as u64 };
        let keyboard_us = kb_end - kb_start;

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

        let clear_start = unsafe { esp_idf_svc::sys::esp_timer_get_time() as u64 };
        if self.current_view.is_need_clear_on_update() {
            self.ui.clear(Rgb565::BLACK);
        }
        let clear_end = unsafe { esp_idf_svc::sys::esp_timer_get_time() as u64 };
        let clear_us = clear_end - clear_start;

        let view_draw_start = unsafe { esp_idf_svc::sys::esp_timer_get_time() as u64 };
        self.current_view.draw(&mut self.ui);
        let view_draw_end = unsafe { esp_idf_svc::sys::esp_timer_get_time() as u64 };
        let view_draw_us = view_draw_end - view_draw_start;

        let top_line_start = unsafe { esp_idf_svc::sys::esp_timer_get_time() as u64 };
        if self.current_view.is_need_top_line() {
            self.ui.draw_top_line(&self.hal.keyboard_state.input_state, &self.hal.keyboard_state.pressed);
        }
        let top_line_end = unsafe { esp_idf_svc::sys::esp_timer_get_time() as u64 };
        let top_line_us = top_line_end - top_line_start;

        let draw_us = view_draw_us + top_line_us;

        let flip_start = unsafe { esp_idf_svc::sys::esp_timer_get_time() as u64 };
        self.ui.flip_buffer();
        let flip_end = unsafe { esp_idf_svc::sys::esp_timer_get_time() as u64 };
        let flip_us = flip_end - flip_start;
        let frame_end = unsafe { esp_idf_svc::sys::esp_timer_get_time() as u64 };
        let frame_us = frame_end - frame_start;

        self.frame_counter = self.frame_counter.wrapping_add(1);
        self.keyboard_us_acc += keyboard_us;
        self.draw_us_acc += draw_us;
        self.clear_us_acc += clear_us;
        self.view_draw_us_acc += view_draw_us;
        self.top_line_us_acc += top_line_us;
        self.flip_us_acc += flip_us;
        self.frame_us_acc += frame_us;

        //if self.frame_counter >= 60 {
            let n = self.frame_counter as u64;
            let keyboard_avg = self.keyboard_us_acc / n;
            let draw_avg = self.draw_us_acc / n;
            let clear_avg = self.clear_us_acc / n;
            let view_draw_avg = self.view_draw_us_acc / n;
            let top_line_avg = self.top_line_us_acc / n;
            let flip_avg = self.flip_us_acc / n;
            let frame_avg = self.frame_us_acc / n;
            log::info!(
                "perf: avg us (frame={}, keyboard={}, clear={}, draw_total={}, draw_view={}, draw_top_line={}, flip={})",
                frame_avg,
                keyboard_avg,
                clear_avg,
                draw_avg,
                view_draw_avg,
                top_line_avg,
                flip_avg,
            );
            self.frame_counter = 0;
            self.keyboard_us_acc = 0;
            self.draw_us_acc = 0;
            self.clear_us_acc = 0;
            self.view_draw_us_acc = 0;
            self.top_line_us_acc = 0;
            self.flip_us_acc = 0;
            self.frame_us_acc = 0;
        //}
    }
}