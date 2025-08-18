use std::{cmp, ffi::CString};

use embedded_graphics::{
    pixelcolor::Rgb565,
    prelude::{Point, RgbColor, WebColors},
};
use esp_idf_hal::delay::FreeRtos;
use esp_idf_sys::{vTaskDelete, xTaskCreatePinnedToCore, BaseType_t, TaskHandle_t};
use u8g2_fonts::types::VerticalPosition;

use crate::{
    cardputer_hal::{
        cardputer_hal::{CardputerHal, KeyboardState},
        input::{
            keyboard::{InputLanguage, PressedSymbol},
            keyboard_io::{KeyEvent, Scancode},
        },
    },
    logic::{view_manager::CardputerView, views::{start::StartView, UiLineElement}},
    ui::cardworder_ui::{CardFont, CardworderUi, ThemeColor},
};

use crate::logic::views::UiLineType;

enum MainMenuOption {
    ConnectWifi,
    UpdateNtp,
    AdditionalInfo
}

pub struct MainMenuView {
    show_fps: bool,
    options: Vec<MainMenuOption>,
    current_item_idx: usize,
    counter: u8,
    lang: InputLanguage,
    timer_task_handler: Option<TaskHandle_t>,
    scroll_offset: u32, // New: scroll offset for the menu
}

impl Default for MainMenuView {
    fn default() -> Self {
        Self {
            show_fps: false,
            options: vec![
                MainMenuOption::ConnectWifi, MainMenuOption::UpdateNtp, MainMenuOption::AdditionalInfo,
                MainMenuOption::ConnectWifi, MainMenuOption::UpdateNtp, MainMenuOption::AdditionalInfo
                ],
            current_item_idx: 0,
            counter: 0,
            lang: InputLanguage::En,
            timer_task_handler: None,
            scroll_offset: 0,
        }
    }
}

fn get_option_text(option: &MainMenuOption, lang: InputLanguage) -> &'static str {
    match (lang, option) {
        (InputLanguage::En, MainMenuOption::ConnectWifi) => "Connect Wi-Fi",
        (InputLanguage::En, MainMenuOption::UpdateNtp) => "Update time by NTP",
        (InputLanguage::Ru, MainMenuOption::ConnectWifi) => "Подключить Wi-Fi",
        (InputLanguage::Ru, MainMenuOption::UpdateNtp) => "Обновить время по NTP",
        (InputLanguage::En, MainMenuOption::AdditionalInfo) => "Additional info",
        (InputLanguage::Ru, MainMenuOption::AdditionalInfo) => "Дополнительная информация",
    }
}

fn get_option_icon_text(option: &MainMenuOption) -> char {
    match option {
        MainMenuOption::ConnectWifi => '\u{25A}',
        MainMenuOption::UpdateNtp => '\u{158}',
        MainMenuOption::AdditionalInfo => '\u{1d5}',
    }
}

unsafe extern "C" fn update_counter(arg: *mut core::ffi::c_void) {
    let main_menu_view = &mut *(arg as *mut MainMenuView);
    loop {
        main_menu_view.counter = main_menu_view.counter.wrapping_add(1);
        log::info!("Counter updated: {}", main_menu_view.counter);
        FreeRtos::delay_ms(50);
    }
}

impl CardputerView for MainMenuView {
    fn is_need_top_line(&self) -> bool {
        true
    }

    fn is_need_clear_on_update(&self) -> bool {
        true
    }

    /// Build the menu as a list of UiLineType
    fn form(&mut self) -> Vec<UiLineType> {
        self.options.iter().enumerate().map(|(idx, o)| {
            let color = if self.current_item_idx == idx {
                ThemeColor::Selected
            } else {
                ThemeColor::Text
            };
            UiLineType::Elements(vec![
                UiLineElement::Icon(get_option_icon_text(&o), CardFont::IconsHuge, color),
                UiLineElement::Spacer(8),
                UiLineElement::Text(get_option_text(&o, self.lang), CardFont::Medium, VerticalPosition::Center, color)
            ])
        }).collect()
    }

    fn init(&mut self, hal: &mut CardputerHal<'_>, ui: &mut CardworderUi<'_>) {
        let mut task_id: TaskHandle_t = core::ptr::null_mut();
        let task_name = CString::new("main_menu_update_counter").unwrap();
        let self_ptr: *mut MainMenuView = self as *mut _;
        unsafe {
            xTaskCreatePinnedToCore(
                Some(update_counter),
                task_name.as_ptr(),
                4096,
                self_ptr as *mut core::ffi::c_void,
                10,
                &mut task_id,
                1,
            )
        };
        self.timer_task_handler = Some(task_id);
    }

    fn destruct(&mut self) {
        match self.timer_task_handler {
            Some(task_handler) => unsafe { vTaskDelete(task_handler) },
            _ => {}
        }
    }

    fn update(&mut self, keyboard_state: &KeyboardState) -> Option<Box<dyn CardputerView>> {
        self.lang = keyboard_state.input_state.lang;
        match (keyboard_state.input_state.opt_pressed, keyboard_state.key) {
            (true, Some((KeyEvent::Pressed, Scancode::F))) => {
                self.show_fps = !self.show_fps;
            }
            _ => {}
        }

        match keyboard_state.pressed {
            Some((KeyEvent::Pressed, PressedSymbol::ArrowDown)) => {
                self.current_item_idx = (self.current_item_idx + 1) % self.options.len();
                // Simple scroll down by estimated line height (will be refined in draw)
                self.scroll_offset = self.scroll_offset.saturating_add(20);
            }
            Some((KeyEvent::Pressed, PressedSymbol::ArrowUp)) => {
                self.current_item_idx = (self.current_item_idx + self.options.len() - 1) % self.options.len();
                // Simple scroll up by estimated line height (will be refined in draw)
                self.scroll_offset = self.scroll_offset.saturating_sub(20);
            }
            Some((KeyEvent::Pressed, PressedSymbol::Enter)) => {
                match self.options[self.current_item_idx] {
                    MainMenuOption::ConnectWifi => {
                        return Some(Box::new(StartView{}));
                    }
                    _ => {}
                }
            }
            _ => {}
        }

        None
    }

    fn draw(&mut self, ui: &mut CardworderUi<'_>) {
        use crate::logic::views::render::{compose_form, render_visible_lines};
        let lines = self.form();
        let composed = compose_form(lines.as_slice(), self.scroll_offset, 135, ui); // 135 = viewport height
        render_visible_lines(&composed, ui);
        ui.show_fps = self.show_fps;
    }
}
