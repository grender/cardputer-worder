use embedded_graphics::pixelcolor::Rgb565;
use embedded_graphics::prelude::WebColors;
use embedded_graphics::prelude::{Point, Size};
use embedded_graphics::primitives::Rectangle;
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
    ui::cardworder_ui::{CardFont, CardworderUi, ThemeColor, TOP_BAR_HEIGHT},
};

use crate::logic::views::UiLineType;

enum MainMenuOption {
    ConnectWifi,
    UpdateNtp,
    AdditionalInfo,
}

pub struct MainMenuView {
    show_fps: bool,
    options: Vec<MainMenuOption>,
    current_item_idx: usize,
    lang: InputLanguage,
}

impl Default for MainMenuView {
    fn default() -> Self {
        Self {
            show_fps: false,
            options: vec![
                MainMenuOption::ConnectWifi, MainMenuOption::UpdateNtp, MainMenuOption::AdditionalInfo,
                MainMenuOption::ConnectWifi, MainMenuOption::UpdateNtp, MainMenuOption::AdditionalInfo,
                MainMenuOption::ConnectWifi, MainMenuOption::UpdateNtp, MainMenuOption::AdditionalInfo
                ],
            current_item_idx: 0,
            lang: InputLanguage::En,
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
        (InputLanguage::Ru, MainMenuOption::AdditionalInfo) => "Дополнительная информация LONG LONG LONG LONG LONG LONG",
    }
}

fn get_option_icon_text(option: &MainMenuOption) -> char {
    match option {
        MainMenuOption::ConnectWifi => '\u{25A}',
        MainMenuOption::UpdateNtp => '\u{158}',
        MainMenuOption::AdditionalInfo => '\u{1d5}',
    }
}

impl CardputerView for MainMenuView {
    fn is_need_top_line(&self) -> bool {
        true
    }

    fn is_need_clear_on_update(&self) -> bool {
        false
    }

    /// Build the menu as a list of UiLineType
    fn form(&mut self) -> Vec<UiLineType> {
        self.options
            .iter()
            .enumerate()
            .map(|(idx, o)| {
                let color = if self.current_item_idx == idx {
                    ThemeColor::Selected
                } else {
                    ThemeColor::Text
                };
                UiLineType::Elements(vec![
                    UiLineElement::Icon(get_option_icon_text(o), CardFont::IconsHuge, color),
                    UiLineElement::Spacer(8),
                    UiLineElement::Text(
                        get_option_text(o, self.lang),
                        CardFont::Medium,
                        VerticalPosition::Center,
                        color,
                    ),
                ])
            })
            .collect()
    }

    fn init(&mut self, _hal: &mut CardputerHal<'_>, _ui: &mut CardworderUi<'_>) {}

    fn destruct(&mut self) {}

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
            }
            Some((KeyEvent::Pressed, PressedSymbol::ArrowUp)) => {
                self.current_item_idx =
                    (self.current_item_idx + self.options.len() - 1) % self.options.len();
            }
            Some((KeyEvent::Pressed, PressedSymbol::Enter)) => {
                match self.options[self.current_item_idx] {
                    MainMenuOption::ConnectWifi => {
                        return Some(Box::new(StartView {}));
                    }
                    _ => {}
                }
            }
            _ => {}
        }

        None
    }

    fn draw(&mut self, ui: &mut CardworderUi<'_>) {
        use crate::logic::views::render::{compose_scrolled_form, render_visible_lines};
        const SCREEN_HEIGHT: u32 = 135;
        let viewport_height = SCREEN_HEIGHT - TOP_BAR_HEIGHT;
        let content = Rectangle::new(
            Point::new(0, TOP_BAR_HEIGHT as i32),
            Size::new(240, viewport_height),
        );
        ui.fill_rect(content, Rgb565::CSS_BLACK);

        let lines = self.form();
        let composed = compose_scrolled_form(
            lines.as_slice(),
            self.current_item_idx,
            viewport_height,
            TOP_BAR_HEIGHT as i32,
            ui,
        );
        render_visible_lines(&composed, lines.as_slice(), ui);
        ui.show_fps = self.show_fps;
    }
}
