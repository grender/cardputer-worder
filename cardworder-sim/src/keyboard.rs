use cardworder_core::input::keyboard::{InputLanguage, InputState, PressedSymbol};
use cardworder_core::input::keyboard_io::{KeyEvent, Scancode};
use cardworder_core::types::KeyMsg;
use sdl2::event::Event;
use sdl2::keyboard::{Keycode, Mod};

pub struct SimKeyboard {
    input_state: InputState,
}

impl SimKeyboard {
    pub fn new() -> Self {
        SimKeyboard {
            input_state: InputState {
                ctrl_pressed: false,
                shift_pressed: false,
                opt_pressed: false,
                alt_pressed: false,
                fn_pressed: false,
                fn_locked: true,
                shift_locked: false,
                fn_last_release_us: 0,
                shift_last_release_us: 0,
                lang: InputLanguage::En,
            },
        }
    }

    pub fn handle_event(&mut self, event: &Event) -> Option<KeyMsg> {
        match event {
            Event::KeyDown { keycode: Some(kc), .. } => {
                // Step 1: update modifier state (calls eat_keys for modifier scancodes,
                // which also handles double-press lock timing for Fn/Shift).
                self.update_modifiers(*kc, KeyEvent::Pressed);

                // Step 2: resolve to (Scancode, PressedSymbol).
                let (sc, sym) = self.resolve(*kc, KeyEvent::Pressed);

                log::info!(
                    "sim key {:?}: scancode={:?} symbol={:?}  [{}]",
                    kc, sc, sym, format_state(&self.input_state)
                );

                Some(KeyMsg {
                    key: sc.map(|s| (KeyEvent::Pressed, s)),
                    input_state: self.input_state,
                    pressed: sym.map(|s| (KeyEvent::Pressed, s)),
                })
            }
            Event::KeyUp { keycode: Some(kc), .. } => {
                self.update_modifiers(*kc, KeyEvent::Released);
                if is_modifier(*kc) {
                    log::info!(
                        "sim release {:?}  [{}]",
                        kc, format_state(&self.input_state)
                    );
                    Some(KeyMsg {
                        key: None,
                        input_state: self.input_state,
                        pressed: None,
                    })
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    /// Delegate modifier key events to `InputState::eat_keys` so double-press
    /// lock/unlock (300 ms window) uses the same timing logic as on hardware.
    fn update_modifiers(&mut self, kc: Keycode, event: KeyEvent) {
        let sc = modifier_scancode(kc);
        if let Some(sc) = sc {
            self.input_state.eat_keys(event, sc);
        }
    }

    /// Resolve an SDL keycode to `(Option<Scancode>, Option<PressedSymbol>)`.
    ///
    /// For dedicated sim keys (arrows, Enter, Esc) we return fixed values.
    /// For all other keys we obtain the Scancode and call `eat_keys` — exactly as
    /// the hardware does — so language, shift, Fn-combos all work identically.
    fn resolve(&mut self, kc: Keycode, event: KeyEvent) -> (Option<Scancode>, Option<PressedSymbol>) {
        // Dedicated sim keys that have no hardware Scancode equivalent
        match kc {
            Keycode::Up    => return (Some(Scancode::Tab),      Some(PressedSymbol::ArrowUp)),
            Keycode::Down  => return (Some(Scancode::Space),    Some(PressedSymbol::ArrowDown)),
            Keycode::Left  => return (Some(Scancode::Fn),       Some(PressedSymbol::ArrowLeft)),
            Keycode::Right => return (Some(Scancode::Ctrl),     Some(PressedSymbol::ArrowRight)),
            Keycode::Escape => return (Some(Scancode::Backspace), Some(PressedSymbol::Esc)),
            _ => {}
        }

        // Keys with a hardware Scancode: let eat_keys produce the correct symbol
        // (respects current lang, shift, Fn state — same as hardware).
        if let Some(sc) = keycode_to_scancode(kc) {
            let sym = self.input_state.eat_keys(event, sc);
            return (Some(sc), sym);
        }

        // Modifier-only keys (F1/F2/Shift/etc.) — already handled in update_modifiers
        (None, None)
    }
}

fn is_modifier(kc: Keycode) -> bool {
    modifier_scancode(kc).is_some()
}

fn modifier_scancode(kc: Keycode) -> Option<Scancode> {
    match kc {
        Keycode::LShift | Keycode::RShift => Some(Scancode::Shift),
        Keycode::LCtrl  | Keycode::RCtrl  => Some(Scancode::Ctrl),
        Keycode::LAlt   | Keycode::RAlt   => Some(Scancode::Alt),
        Keycode::F1 => Some(Scancode::Fn),
        Keycode::F2 => Some(Scancode::Opt),
        _ => None,
    }
}

/// Map SDL keycodes to hardware Scancodes.
/// Letters/numbers/punctuation map to the same physical position on the
/// Cardputer keyboard so that `eat_keys` produces the right symbol
/// (including Cyrillic when lang=Ru).
fn keycode_to_scancode(kc: Keycode) -> Option<Scancode> {
    match kc {
        Keycode::A => Some(Scancode::A),
        Keycode::B => Some(Scancode::B),
        Keycode::C => Some(Scancode::C),
        Keycode::D => Some(Scancode::D),
        Keycode::E => Some(Scancode::E),
        Keycode::F => Some(Scancode::F),
        Keycode::G => Some(Scancode::G),
        Keycode::H => Some(Scancode::H),
        Keycode::I => Some(Scancode::I),
        Keycode::J => Some(Scancode::J),
        Keycode::K => Some(Scancode::K),
        Keycode::L => Some(Scancode::L),
        Keycode::M => Some(Scancode::M),
        Keycode::N => Some(Scancode::N),
        Keycode::O => Some(Scancode::O),
        Keycode::P => Some(Scancode::P),
        Keycode::Q => Some(Scancode::Q),
        Keycode::R => Some(Scancode::R),
        Keycode::S => Some(Scancode::S),
        Keycode::T => Some(Scancode::T),
        Keycode::U => Some(Scancode::U),
        Keycode::V => Some(Scancode::V),
        Keycode::W => Some(Scancode::W),
        Keycode::X => Some(Scancode::X),
        Keycode::Y => Some(Scancode::Y),
        Keycode::Z => Some(Scancode::Z),
        Keycode::Num0 => Some(Scancode::_0),
        Keycode::Num1 => Some(Scancode::_1),
        Keycode::Num2 => Some(Scancode::_2),
        Keycode::Num3 => Some(Scancode::_3),
        Keycode::Num4 => Some(Scancode::_4),
        Keycode::Num5 => Some(Scancode::_5),
        Keycode::Num6 => Some(Scancode::_6),
        Keycode::Num7 => Some(Scancode::_7),
        Keycode::Num8 => Some(Scancode::_8),
        Keycode::Num9 => Some(Scancode::_9),
        Keycode::Space     => Some(Scancode::Space),
        Keycode::Return | Keycode::KpEnter => Some(Scancode::Enter),
        Keycode::Backspace => Some(Scancode::Backspace),
        Keycode::Delete    => Some(Scancode::Backspace),
        Keycode::Period    => Some(Scancode::Period),
        Keycode::Comma     => Some(Scancode::Comma),
        Keycode::Slash     => Some(Scancode::Slash),
        Keycode::Semicolon => Some(Scancode::Semicolon),
        Keycode::Quote     => Some(Scancode::Quote),
        Keycode::Minus     => Some(Scancode::Underscore),
        Keycode::Equals    => Some(Scancode::Equal),
        Keycode::Backslash => Some(Scancode::BackSlash),
        Keycode::LeftBracket  => Some(Scancode::LeftSquareBracket),
        Keycode::RightBracket => Some(Scancode::RightSquareBracket),
        Keycode::Backquote => Some(Scancode::Tilde),
        _ => None,
    }
}

fn format_state(s: &InputState) -> String {
    let mut parts = Vec::new();
    if s.ctrl_pressed  { parts.push("Ctrl");  }
    if s.shift_pressed { parts.push("Shift"); }
    if s.fn_pressed    { parts.push("Fn");    }
    if s.opt_pressed   { parts.push("Opt");   }
    if s.alt_pressed   { parts.push("Alt");   }
    if s.fn_locked     { parts.push("FnLck"); }
    if s.shift_locked  { parts.push("ShLck"); }
    let lang = match s.lang { InputLanguage::En => "EN", InputLanguage::Ru => "RU" };
    if parts.is_empty() { format!("lang={}", lang) }
    else { format!("{} lang={}", parts.join("+"), lang) }
}

/// Print the simulator keymap to the log at startup.
pub fn log_keymap() {
    log::info!("=== Simulator keymap ===");
    log::info!("  Navigation:");
    log::info!("    Arrow keys        → Up / Down / Left / Right");
    log::info!("    Enter             → Confirm / Select");
    log::info!("    Backspace         → Delete previous char");
    log::info!("    Delete            → Delete (Del)");
    log::info!("    Escape            → Esc / Back");
    log::info!("  Modifiers (hold):");
    log::info!("    LShift / RShift   → Shift (uppercase / symbols)");
    log::info!("    LCtrl  / RCtrl    → Ctrl");
    log::info!("    LAlt   / RAlt     → Alt");
    log::info!("    F1                → Fn  (arrows via Fn+; . , /)");
    log::info!("    F2                → Opt (e.g. Opt+F = toggle FPS)");
    log::info!("  Lock / unlock (double-press within 300 ms):");
    log::info!("    F1 × 2            → toggle Fn lock  (default: ON)");
    log::info!("    Shift × 2         → toggle Shift lock");
    log::info!("  Language:");
    log::info!("    Ctrl + Space      → switch EN ↔ RU");
    log::info!("  Misc:");
    log::info!("    F10               → quit simulator");
    log::info!("========================");
}
