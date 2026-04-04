/// Platform-agnostic key event types.
/// The `CardputerKeyboard` hardware driver lives in `cardworder/src/cardputer_hal/`.

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Scancode {
    Space = 6,
    Period = 5,
    M = 4,
    B = 3,
    C = 2,
    Z = 1,
    Opt = 0,

    Enter = 13,
    Semicolon = 12,
    K = 11,
    H = 10,
    F = 9,
    S = 8,
    Shift = 7,

    BackSlash = 20,
    LeftSquareBracket = 19,
    O = 18,
    U = 17,
    T = 16,
    E = 15,
    Q = 14,

    Backspace = 27,
    Underscore = 26,
    _9 = 25,
    _7 = 24,
    _5 = 23,
    _3 = 22,
    _1 = 21,

    Slash = 34,
    Comma = 33,
    N = 32,
    V = 31,
    X = 30,
    Alt = 29,
    Ctrl = 28,

    Quote = 41,
    L = 40,
    J = 39,
    G = 38,
    D = 37,
    A = 36,
    Fn = 35,

    RightSquareBracket = 48,
    P = 47,
    I = 46,
    Y = 45,
    R = 44,
    W = 43,
    Tab = 42,

    Equal = 55,
    _0 = 54,
    _8 = 53,
    _6 = 52,
    _4 = 51,
    _2 = 50,
    Tilde = 49,
}

#[derive(Debug, Copy, Clone)]
pub enum KeyEvent {
    Pressed,
    Released,
}
