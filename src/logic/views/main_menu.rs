use embedded_graphics::{pixelcolor::Rgb565, prelude::{RgbColor, WebColors}};

use crate::{cardputer_hal::{cardputer_hal::{CardputerHal, KeyboardState}, input::{keyboard::PressedSymbol, keyboard_io::{KeyEvent, Scancode}}}, logic::{view_manager::CardputerView, views::start::StartView}, ui::cardworder_ui::CardworderUi};

enum MainMenuOption {
    Nothing,
    ConnectWifiAndUpdateNtp,
}

pub struct MainMenuView {
    show_fps: bool,
    current_option: Option<MainMenuOption>,    
}

impl Default for MainMenuView {
    fn default() -> Self {
        Self { show_fps: false, current_option: None }
    }
}

impl CardputerView for MainMenuView {
    fn is_need_top_line(&self) -> bool {
        true
    }

    fn is_need_clear_on_update(&self) -> bool {
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

        match keyboard_state.pressed {
            Some((KeyEvent::Pressed, PressedSymbol::ArrowDown)) => {
                self.current_option = Some(MainMenuOption::ConnectWifiAndUpdateNtp);
            }
            Some((KeyEvent::Pressed, PressedSymbol::ArrowUp)) => {
                self.current_option = Some(MainMenuOption::Nothing);
            }
            Some((KeyEvent::Pressed, PressedSymbol::Enter)) => {
                match self.current_option { 
                    Some(MainMenuOption::ConnectWifiAndUpdateNtp) => {
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
        if matches!(self.current_option, Some(MainMenuOption::Nothing)) {
            ui.draw_text_huge("> Nothing", 0, 8, Rgb565::CSS_LIGHT_BLUE);
        } else {
            ui.draw_text_huge("  Nothing", 0, 8, Rgb565::CSS_WHITE);
        }
        
        if matches!(self.current_option, Some(MainMenuOption::ConnectWifiAndUpdateNtp)) {
            ui.draw_text_huge("> Connect Wifi and Update Ntp", 0, 21, Rgb565::CSS_LIGHT_BLUE);
        } else {
            ui.draw_text_huge("  Connect Wifi and Update Ntp", 0, 21, Rgb565::WHITE);
        }

        ui.show_fps = self.show_fps;
    }
}