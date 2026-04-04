use esp_idf_hal::gpio::{Input, Output, PinDriver};

pub use cardworder_core::input::keyboard_io::{KeyEvent, Scancode};

type KeyboardState = [u8; 8];
pub struct CardputerKeyboard<'a> {
    mux: [PinDriver<'a, Output>; 3],
    columns: [PinDriver<'a, Input>; 7],
    state: KeyboardState,
}

impl<'a> CardputerKeyboard<'a> {
    pub fn new(
        mux: [PinDriver<'a, Output>; 3],
        columns: [PinDriver<'a, Input>; 7],
    ) -> Self {
        Self {
            mux,
            columns,
            state: [0; 8],
        }
    }

    pub fn init(&mut self) {
        // Pull-up is configured in CardputerHal::new via PinDriver::input(..., Pull::Up).
    }

    pub fn read_columns(&self) -> u8 {
        let mut result = 0;
        for (i, column) in self.columns.iter().enumerate() {
            if column.is_low() {
                result |= 1 << i;
            }
        }
        result
    }

    pub fn set_mux(&mut self, index: u8) {
        for i in 0..3 {
            if index & (1 << i) != 0 {
                self.mux[i].set_high().unwrap();
            } else {
                self.mux[i].set_low().unwrap();
            }
        }
    }

    /// Reads the raw state of the keyboard.
    pub fn read_keys_raw(&mut self) -> KeyboardState {
        let mut result = [0; 8];
        for i in 0..8 {
            self.set_mux(i);
            result[i as usize] = self.read_columns();
        }
        result
    }

    /// Reads the state of the keyboard and returns a list of pressed keys.
    pub fn read_keys(&mut self) -> Vec<Scancode> {
        log::info!("keyboard_io: read_keys — called");
        let raw = self.read_keys_raw();
        let mut result = Vec::new();
        for (i, byte) in raw.iter().enumerate() {
            for j in 0..8 {
                if byte & (1 << j) != 0 {
                    result.push(KEY_MAP[i * 7 + j]);
                }
            }
        }
        result
    }

    /// Returns the derivative of the keyboard state since the last call.
    pub fn read_events_raw(&mut self) -> KeyboardState {
        let keys = self.read_keys_raw();
        let mut result = [0; 8];
        for i in 0..8 {
            result[i] = keys[i] ^ self.state[i];
            self.state[i] = keys[i];
        }
        result
    }

    /// Returns the Pressed/Released events since the last call.
    /// WARNING: This assumes that only one key is pressed / released at a time.
    pub fn read_events(&mut self) -> Option<(KeyEvent, Scancode)> {
        let raw = self.read_events_raw();
        for (i, byte) in raw.iter().enumerate() {
            for j in 0..8 {
                if byte & (1 << j) != 0 {
                    return Some((
                        if self.state[i] & (1 << j) == 0 {
                            KeyEvent::Released
                        } else {
                            KeyEvent::Pressed
                        },
                        KEY_MAP[i * 7 + j],
                    ));
                }
            }
        }

        None
    }
}

// Scancode and KeyEvent are re-exported from cardworder_core::input::keyboard_io above.

const KEY_MAP: [Scancode; 56] = [
    Scancode::Opt,
    Scancode::Z,
    Scancode::C,
    Scancode::B,
    Scancode::M,
    Scancode::Period,
    Scancode::Space,
    Scancode::Shift,
    Scancode::S,
    Scancode::F,
    Scancode::H,
    Scancode::K,
    Scancode::Semicolon,
    Scancode::Enter,
    Scancode::Q,
    Scancode::E,
    Scancode::T,
    Scancode::U,
    Scancode::O,
    Scancode::LeftSquareBracket,
    Scancode::BackSlash,
    Scancode::_1,
    Scancode::_3,
    Scancode::_5,
    Scancode::_7,
    Scancode::_9,
    Scancode::Underscore,
    Scancode::Backspace,
    Scancode::Ctrl,
    Scancode::Alt,
    Scancode::X,
    Scancode::V,
    Scancode::N,
    Scancode::Comma,
    Scancode::Slash,
    Scancode::Fn,
    Scancode::A,
    Scancode::D,
    Scancode::G,
    Scancode::J,
    Scancode::L,
    Scancode::Quote,
    Scancode::Tab,
    Scancode::W,
    Scancode::R,
    Scancode::Y,
    Scancode::I,
    Scancode::P,
    Scancode::RightSquareBracket,
    Scancode::Tilde,
    Scancode::_2,
    Scancode::_4,
    Scancode::_6,
    Scancode::_8,
    Scancode::_0,
    Scancode::Equal,
];
