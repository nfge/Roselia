use crate::{keyboard::{key, key_state::{self, KeyState}}, vga::writer::Writer};

pub(self) struct Key {
    keycode: u8,
    letter: char,
}
pub struct KeyTable;

static SCANCODE_TABLE: &[Key] = &[
    Key { keycode: 0x1E, letter: 'a'}, // standard
    Key { keycode: 0x30, letter: 'b'},
    Key { keycode: 0x2E, letter: 'c'},
    Key { keycode: 0x20, letter: 'd'},
    Key { keycode: 0x12, letter: 'e'},
    Key { keycode: 0x21, letter: 'f'},
    Key { keycode: 0x22, letter: 'g'},
    Key { keycode: 0x23, letter: 'h'},
    Key { keycode: 0x17, letter: 'i'},
    Key { keycode: 0x24, letter: 'j'},
    Key { keycode: 0x25, letter: 'k'},
    Key { keycode: 0x26, letter: 'l'},
    Key { keycode: 0x32, letter: 'm'},
    Key { keycode: 0x31, letter: 'n'},
    Key { keycode: 0x18, letter: 'o'},
    Key { keycode: 0x19, letter: 'p'},
    Key { keycode: 0x10, letter: 'q'},
    Key { keycode: 0x13, letter: 'r'},
    Key { keycode: 0x1F, letter: 's'},
    Key { keycode: 0x14, letter: 't'},
    Key { keycode: 0x16, letter: 'u'},
    Key { keycode: 0x2F, letter: 'v'},
    Key { keycode: 0x11, letter: 'w'},
    Key { keycode: 0x2D, letter: 'x'},
    Key { keycode: 0x15, letter: 'y'},
    Key { keycode: 0x2C, letter: 'z'},
    Key { keycode: 0x39, letter: ' '}, // special
    Key { keycode: 0x1C, letter: '\n'}, 
    Key { keycode: 0x0E, letter: '\x08'},
    // Key { keycode: 0x2A, letter: '\0'},
    Key { keycode: 0x02, letter: '1'}, // numbers
    Key { keycode: 0x03, letter: '2'},
    Key { keycode: 0x04, letter: '3'},
    Key { keycode: 0x05, letter: '4'},
    Key { keycode: 0x06, letter: '5'},
    Key { keycode: 0x07, letter: '6'},
    Key { keycode: 0x08, letter: '7'},
    Key { keycode: 0x09, letter: '8'},
    Key { keycode: 0x0A, letter: '9'},
    Key { keycode: 0x0B, letter: '0'},
    Key { keycode: 0x4B, letter: '\0'}, // arrows
    Key { keycode: 0x4D, letter: '\0'},
    Key { keycode: 0x48, letter: '\0'},
    Key { keycode: 0x50, letter: '\0'},
];

impl KeyTable {
    pub fn get_letter(writer: &mut Writer,keycode: u8, state: &mut KeyState) -> char {
        for key in SCANCODE_TABLE {
            if key.keycode == keycode {
                if state.shift {
                    return key.letter.to_ascii_uppercase();
                } else {
                    return key.letter;
                }
            }
        }
        return '\0';
    }
    // fn get_arrows(writer: &mut Writer,keycode: u8) {
    //     match keycode {
    //         0x4B => writer.extern_move_cursor(0, 1),
    //         0x4D => writer.extern_move_cursor(0, -1),
    //         0x48 => writer.extern_move_cursor(-1, 0),
    //         0x50 => writer.extern_move_cursor(1, 0),
    //         _ => { panic!("Invalid arrow keycode: {}", keycode) },
    //     }
    // }
}