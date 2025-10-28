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
    AdditionalInfo,
    Settings,
    About,
    Help,
    Exit,
    Test1,
    Test2,
    Test3,
    Test4,
    Test5,
}

pub struct MainMenuView<'a> {
    show_fps: bool,
    options: Vec<MainMenuOption>,
    current_item_idx: usize,
    counter: u8,
    lang: InputLanguage,
    timer_task_handler: Option<TaskHandle_t>,
    scroll_offset: u32, // New: scroll offset for the menu
    form_lines: Option<Vec<UiLineType<'a>>>,
    input_value: String
}

impl <'a> Default for MainMenuView<'a> {
    fn default() -> Self {
        Self {
            show_fps: false,
            options: vec![
                MainMenuOption::ConnectWifi, 
                MainMenuOption::UpdateNtp, 
                MainMenuOption::AdditionalInfo,
                MainMenuOption::Settings,
                MainMenuOption::About,
                MainMenuOption::Help,
                MainMenuOption::Exit,
                MainMenuOption::Test1,
                MainMenuOption::Test2,
                MainMenuOption::Test3,
                MainMenuOption::Test4,
                MainMenuOption::Test5,
            ],
            current_item_idx: 0,
            counter: 0,
            lang: InputLanguage::En,
            timer_task_handler: None,
            scroll_offset: 0,
            form_lines: None,
            input_value: "".to_string(),
        }
    }
}

fn get_option_text(option: &MainMenuOption, lang: InputLanguage) -> &'static str {
    match (lang, option) {
        (InputLanguage::En, MainMenuOption::ConnectWifi) => "Connect Wi-Fi",
        (InputLanguage::En, MainMenuOption::UpdateNtp) => "Update time by NTP",
        (InputLanguage::En, MainMenuOption::AdditionalInfo) => "Additional info",
        (InputLanguage::En, MainMenuOption::Settings) => "Settings",
        (InputLanguage::En, MainMenuOption::About) => "About",
        (InputLanguage::En, MainMenuOption::Help) => "Help",
        (InputLanguage::En, MainMenuOption::Exit) => "Exit",
        (InputLanguage::En, MainMenuOption::Test1) => "Test Item 1",
        (InputLanguage::En, MainMenuOption::Test2) => "Test Item 2",
        (InputLanguage::En, MainMenuOption::Test3) => "Test Item 3",
        (InputLanguage::En, MainMenuOption::Test4) => "Test Item 4",
        (InputLanguage::En, MainMenuOption::Test5) => "Test Item 5",
        (InputLanguage::Ru, MainMenuOption::ConnectWifi) => "Подключить Wi-Fi",
        (InputLanguage::Ru, MainMenuOption::UpdateNtp) => "Обновить время по NTP",
        (InputLanguage::Ru, MainMenuOption::AdditionalInfo) => "Дополнительная информация",
        (InputLanguage::Ru, MainMenuOption::Settings) => "Настройки",
        (InputLanguage::Ru, MainMenuOption::About) => "О программе",
        (InputLanguage::Ru, MainMenuOption::Help) => "Помощь",
        (InputLanguage::Ru, MainMenuOption::Exit) => "Выход",
        (InputLanguage::Ru, MainMenuOption::Test1) => "Тест 1",
        (InputLanguage::Ru, MainMenuOption::Test2) => "Тест 2",
        (InputLanguage::Ru, MainMenuOption::Test3) => "Тест 3",
        (InputLanguage::Ru, MainMenuOption::Test4) => "Тест 4",
        (InputLanguage::Ru, MainMenuOption::Test5) => "Тест 5",
    }
}

fn get_option_icon_text(option: &MainMenuOption) -> char {
    match option {
        MainMenuOption::ConnectWifi => '\u{25A}',
        MainMenuOption::UpdateNtp => '\u{158}',
        MainMenuOption::AdditionalInfo => '\u{1d5}',
        MainMenuOption::Settings => '\u{25A}',
        MainMenuOption::About => '\u{25A}',
        MainMenuOption::Help => '\u{25A}',
        MainMenuOption::Exit => '\u{25A}',
        MainMenuOption::Test1 => '\u{25A}',
        MainMenuOption::Test2 => '\u{25A}',
        MainMenuOption::Test3 => '\u{25A}',
        MainMenuOption::Test4 => '\u{25A}',
        MainMenuOption::Test5 => '\u{25A}',
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

impl <'a> CardputerView<'a> for MainMenuView<'a> {

    fn is_need_top_line(&self) -> bool {
        true
    }

    fn is_need_clear_on_update(&self) -> bool {
        true
    }

    /// Build the menu as a list of UiLineType
    fn form(&'a self) -> Vec<UiLineType<'a>> {
        let mut lines= self.options.iter().enumerate().map(|(idx, o)| {
            let color = if self.current_item_idx == idx {
                ThemeColor::Selected
            } else {
                ThemeColor::Text
            };
            let r:UiLineType<'a> = UiLineType::Elements(vec![
                UiLineElement::Icon(get_option_icon_text(&o), CardFont::IconsHuge, color),
                UiLineElement::Spacer(8),
                UiLineElement::Text(get_option_text(&o, self.lang), CardFont::Medium, VerticalPosition::Top, color)
            ]);
            r
        }).collect::<Vec<UiLineType<'a>>>();
        
        // Add a test input field to demonstrate the new InputText element
        lines.push(UiLineType::Elements(vec![
            UiLineElement::<'a>::Text("Input: ", CardFont::Medium, VerticalPosition::Top, ThemeColor::Text),
            UiLineElement::<'a>::InputText(&self.input_value, CardFont::Medium, ThemeColor::Selected, 0)
        ]));

        lines
        
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
        let form_lines = self.form();
        self.form_lines = Some(form_lines)
    }

    fn destruct(&mut self) {
        match self.timer_task_handler {
            Some(task_handler) => unsafe { vTaskDelete(task_handler) },
            _ => {}
        }
    }

    fn update(&mut self, keyboard_state: &KeyboardState) -> Option<Box<dyn CardputerView<'a>>> {
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
                // Scroll down by one line height (approximately 20 pixels for menu items)
                self.scroll_offset = self.scroll_offset.saturating_add(20);
            }
            Some((KeyEvent::Pressed, PressedSymbol::ArrowUp)) => {
                self.current_item_idx = (self.current_item_idx + self.options.len() - 1) % self.options.len();
                // Scroll up by one line height
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

    fn draw(&self, ui: &mut CardworderUi<'_>) {
        use crate::logic::views::render::{compose_form, render_visible_lines};
        
        // Clear the screen first
        ui.clear(Rgb565::BLACK);
        let scroll_offset = self.scroll_offset;
        // Compose the form with current scroll offset
        let composed = compose_form(self.form_lines.as_ref().unwrap().as_slice(), scroll_offset, 125, ui); // 125 = viewport height (135 - 10 for top line)
        
        // Render the visible lines and scroll bar
        render_visible_lines(&composed, ui);
        
        // Update FPS display
        ui.show_fps = self.show_fps;
    }
}
