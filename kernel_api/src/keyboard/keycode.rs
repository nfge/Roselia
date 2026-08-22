use super::keyevent::KeyEvent;

#[derive(Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum KeyCode {
    A = 'a' as u32,
    B = 'b' as u32,
    C = 'c' as u32,
    D = 'd' as u32,
    E = 'e' as u32,
    F = 'f' as u32,
    G = 'g' as u32,
    H = 'h' as u32,
    I = 'i' as u32,
    J = 'j' as u32,
    K = 'k' as u32,
    L = 'l' as u32,
    M = 'm' as u32,
    N = 'n' as u32,
    O = 'o' as u32,
    P = 'p' as u32,
    Q = 'q' as u32,
    R = 'r' as u32,
    S = 's' as u32,
    T = 't' as u32,
    U = 'u' as u32,
    V = 'v' as u32,
    W = 'w' as u32,
    X = 'x' as u32,
    Y = 'y' as u32,
    Z = 'z' as u32,

    Key0 = '0' as u32,
    Key1 = '1' as u32,
    Key2 = '2' as u32,
    Key3 = '3' as u32,
    Key4 = '4' as u32,
    Key5 = '5' as u32,
    Key6 = '6' as u32,
    Key7 = '7' as u32,
    Key8 = '8' as u32,
    Key9 = '9' as u32,

    Space = ' ' as u32,
    Enter = '\n' as u32,
    Backspace = '\u{8}' as u32,
    Escape = '\u{1B}' as u32,

    Minus = '-' as u32,
    Equals = '=' as u32,
    Comma = ',' as u32,
    Period = '.' as u32,
    Quote = '\'' as u32,
    LeftBracket = '[' as u32,
    RightBracket = ']' as u32,
    Backslash = '\\' as u32,
    Slash = '/' as u32,
    Semicolon = ';' as u32,
    Colon = ':' as u32,

    ArrowLeft = 0x100,
    ArrowRight = 0x101,
    ArrowUp = 0x102,
    ArrowDown = 0x103,
}

impl KeyCode {
    pub fn to_char(self) -> Option<char> {
        use KeyCode::*;
        match self {
            ArrowUp | ArrowDown | ArrowLeft | ArrowRight => None,
            key => char::from_u32(key as u32),
        }
    }
}

pub fn scancode_to_keycode(scancode: u8) -> Option<KeyCode> {
    Some(match scancode {
        0x1E => KeyCode::A,
        0x30 => KeyCode::B,
        0x2E => KeyCode::C,
        0x20 => KeyCode::D,
        0x12 => KeyCode::E,
        0x21 => KeyCode::F,
        0x22 => KeyCode::G,
        0x23 => KeyCode::H,
        0x17 => KeyCode::I,
        0x24 => KeyCode::J,
        0x25 => KeyCode::K,
        0x26 => KeyCode::L,
        0x32 => KeyCode::M,
        0x31 => KeyCode::N,
        0x18 => KeyCode::O,
        0x19 => KeyCode::P,
        0x10 => KeyCode::Q,
        0x13 => KeyCode::R,
        0x1F => KeyCode::S,
        0x14 => KeyCode::T,
        0x16 => KeyCode::U,
        0x2F => KeyCode::V,
        0x11 => KeyCode::W,
        0x2D => KeyCode::X,
        0x15 => KeyCode::Y,
        0x2C => KeyCode::Z,

        0x39 => KeyCode::Space,
        0x1C => KeyCode::Enter,
        0x0E => KeyCode::Backspace,
        0x01 => KeyCode::Escape,

        0x02 => KeyCode::Key1,
        0x03 => KeyCode::Key2,
        0x04 => KeyCode::Key3,
        0x05 => KeyCode::Key4,
        0x06 => KeyCode::Key5,
        0x07 => KeyCode::Key6,
        0x08 => KeyCode::Key7,
        0x09 => KeyCode::Key8,
        0x0A => KeyCode::Key9,
        0x0B => KeyCode::Key0,

        0x4B => KeyCode::ArrowLeft,
        0x4D => KeyCode::ArrowRight,
        0x48 => KeyCode::ArrowUp,
        0x50 => KeyCode::ArrowDown,

        0x0C => KeyCode::Minus,
        0x0D => KeyCode::Equals,
        0x33 => KeyCode::Comma,
        0x34 => KeyCode::Period,
        0x28 => KeyCode::Quote,
        0x1A => KeyCode::LeftBracket,
        0x1B => KeyCode::RightBracket,
        0x2B => KeyCode::Backslash,
        0x35 => KeyCode::Slash,
        0x27 => KeyCode::Semicolon,

        _ => return None,
    })
}

pub fn key_event_to_char(evt: KeyEvent) -> Option<char> {
    let base = evt.code.to_char()?;
    if !evt.shift {
        return Some(base);
    }

    if base.is_ascii_alphabetic() {
        return Some(base.to_ascii_uppercase());
    }

    Some(match base {
        '1' => '!',
        '2' => '@',
        '3' => '#',
        '4' => '$',
        '5' => '%',
        '6' => '^',
        '7' => '&',
        '8' => '*',
        '9' => '(',
        '0' => ')',
        '-' => '_',
        '=' => '+',
        ',' => '<',
        '.' => '>',
        '/' => '?',
        '\'' => '"',
        '[' => '{',
        ']' => '}',
        '\\' => '|',
        ';' => ':',
        _ => base,
    })
}