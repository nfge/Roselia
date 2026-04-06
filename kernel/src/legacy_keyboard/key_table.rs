use crate::{keyboard::{key, key_state::{self, KeyState}}, vga::writer::Writer};
use super::scancode_table::SCANCODE_TABLE;
pub(super) struct Key {
    pub(super) scancode: u8,
    pub(super) letter: char,
}
pub struct KeyTable;


impl KeyTable {
    pub fn get_letter(writer: &mut Writer,scancode: u8, state: &mut KeyState) -> char {
        for key in SCANCODE_TABLE {
            if key.scancode == scancode {
                if state.shift {
                    return key.letter.to_ascii_uppercase();
                } else {
                    return key.letter;
                }
            }
        }
        return '\0';
    }
    // fn get_arrows(writer: &mut Writer,scancode: u8) {
    //     match scancode {
    //         0x4B => writer.extern_move_cursor(0, 1),
    //         0x4D => writer.extern_move_cursor(0, -1),
    //         0x48 => writer.extern_move_cursor(-1, 0),
    //         0x50 => writer.extern_move_cursor(1, 0),
    //         _ => { panic!("Invalid arrow scancode: {}", scancode) },
    //     }
    // }
}