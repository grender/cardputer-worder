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

    // Cached menu + layout.
    cached_lang: Option<InputLanguage>,
    cached_lines: Vec<UiLineType>,
    cached_composed: Vec<crate::logic::views::render::ComposedForm>,
    prev_item_idx: usize,

    // Draw perf stats for this view only.
    draw_frame_counter: u32,
    draw_total_us_acc: u64,
    draw_rebuild_cache_us_acc: u64,
    draw_recolor_us_acc: u64,
    draw_clear_dirty_us_acc: u64,
    draw_render_us_acc: u64,
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

            cached_lang: None,
            cached_lines: Vec::new(),
            cached_composed: Vec::new(),
            prev_item_idx: 0,
            draw_frame_counter: 0,
            draw_total_us_acc: 0,
            draw_rebuild_cache_us_acc: 0,
            draw_recolor_us_acc: 0,
            draw_clear_dirty_us_acc: 0,
            draw_render_us_acc: 0,
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
        use crate::logic::views::render::{
            compose_scrolled_form, render_visible_lines, ComposedForm,
        };
        const SCREEN_HEIGHT: u32 = 135;
        const PERF_WINDOW: u32 = 60;
        let viewport_height = SCREEN_HEIGHT - TOP_BAR_HEIGHT;
        let draw_start = unsafe { esp_idf_svc::sys::esp_timer_get_time() as u64 };
        let mut rebuild_cache_us = 0_u64;

        // (Re)build cached menu structure + composed layout when language changes.
        let rebuild_cache_start = unsafe { esp_idf_svc::sys::esp_timer_get_time() as u64 };
        if self.cached_lang != Some(self.lang) || self.cached_lines.is_empty() {
            // The menu structure (icons + static text) depends on language, but not on selection.
            self.cached_lines = self
                .options
                .iter()
                .enumerate()
                .map(|(_idx, o)| {
                    UiLineType::Elements(vec![
                        UiLineElement::Icon(
                            get_option_icon_text(o),
                            CardFont::IconsHuge,
                            ThemeColor::Text,
                        ),
                        UiLineElement::Spacer(8),
                        UiLineElement::Text(
                            get_option_text(o, self.lang),
                            CardFont::Medium,
                            VerticalPosition::Center,
                            ThemeColor::Text,
                        ),
                    ])
                })
                .collect();

            // Precompute composed layouts for every possible selection index.
            self.cached_composed = (0..self.cached_lines.len())
                .map(|selected_idx| {
                    compose_scrolled_form(
                        self.cached_lines.as_slice(),
                        selected_idx,
                        viewport_height,
                        TOP_BAR_HEIGHT as i32,
                        ui,
                    )
                })
                .collect::<Vec<ComposedForm>>();

            self.prev_item_idx = self.current_item_idx;
            self.cached_lang = Some(self.lang);
        }
        let rebuild_cache_end = unsafe { esp_idf_svc::sys::esp_timer_get_time() as u64 };
        rebuild_cache_us = rebuild_cache_end - rebuild_cache_start;

        // Update colors in-place to avoid rebuilding line Vecs.
        let recolor_start = unsafe { esp_idf_svc::sys::esp_timer_get_time() as u64 };
        let selected = self.current_item_idx;
        for (idx, line) in self.cached_lines.iter_mut().enumerate() {
            let color = if idx == selected {
                ThemeColor::Selected
            } else {
                ThemeColor::Text
            };
            if let UiLineType::Elements(ref mut elements) = line {
                for el in elements.iter_mut() {
                    match el {
                        UiLineElement::Icon(_, _, c) => *c = color,
                        UiLineElement::Text(_, _, _, c) => *c = color,
                        UiLineElement::Spacer(_) | UiLineElement::Filler => {}
                    }
                }
            }
        }
        let recolor_end = unsafe { esp_idf_svc::sys::esp_timer_get_time() as u64 };
        let recolor_us = recolor_end - recolor_start;

        // Minimal redraw: clear only rectangles that can change between prev and current.
        let composed_now = &self.cached_composed[self.current_item_idx];
        let clear_dirty_start = unsafe { esp_idf_svc::sys::esp_timer_get_time() as u64 };
//        let composed_prev = &self.cached_composed[self.prev_item_idx];
//        let composed_now = &self.cached_composed[self.current_item_idx];
//        for line in composed_prev.lines.iter() {
//            ui.fill_rect(line.rect, Rgb565::CSS_BLACK);
//        }
//        for line in composed_now.lines.iter() {
//            ui.fill_rect(line.rect, Rgb565::CSS_BLACK);
//        }
//
//        // Always clear scrollbar track area; `render_visible_lines` will redraw it if needed.
//        let scrollbar_width: i32 = 4;
//        let track_rect = Rectangle::new(
//            Point::new(240 - scrollbar_width, TOP_BAR_HEIGHT as i32),
//            Size::new(scrollbar_width as u32, viewport_height),
//        );
//        ui.fill_rect(track_rect, Rgb565::CSS_BLACK);
        let clear_dirty_end = unsafe { esp_idf_svc::sys::esp_timer_get_time() as u64 };
        let clear_dirty_us = clear_dirty_end - clear_dirty_start;

        let render_start = unsafe { esp_idf_svc::sys::esp_timer_get_time() as u64 };
        render_visible_lines(composed_now, self.cached_lines.as_slice(), ui);
        ui.show_fps = self.show_fps;
        let render_end = unsafe { esp_idf_svc::sys::esp_timer_get_time() as u64 };
        let render_us = render_end - render_start;

        self.prev_item_idx = self.current_item_idx;

        let draw_end = unsafe { esp_idf_svc::sys::esp_timer_get_time() as u64 };
        let draw_total_us = draw_end - draw_start;
        self.draw_frame_counter = self.draw_frame_counter.wrapping_add(1);
        self.draw_total_us_acc += draw_total_us;
        self.draw_rebuild_cache_us_acc += rebuild_cache_us;
        self.draw_recolor_us_acc += recolor_us;
        self.draw_clear_dirty_us_acc += clear_dirty_us;
        self.draw_render_us_acc += render_us;

        //if self.draw_frame_counter >= PERF_WINDOW {
            let n = self.draw_frame_counter as u64;
            log::info!(
                "perf main_menu::draw avg us (total={}, rebuild_cache={}, recolor={}, clear_dirty={}, render={})",
                self.draw_total_us_acc / n,
                self.draw_rebuild_cache_us_acc / n,
                self.draw_recolor_us_acc / n,
                self.draw_clear_dirty_us_acc / n,
                self.draw_render_us_acc / n,
            );
            self.draw_frame_counter = 0;
            self.draw_total_us_acc = 0;
            self.draw_rebuild_cache_us_acc = 0;
            self.draw_recolor_us_acc = 0;
            self.draw_clear_dirty_us_acc = 0;
            self.draw_render_us_acc = 0;
        //}
    }
}
