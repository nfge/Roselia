use super::{scancode_table::SCANCODE_TABLE, KeyState};
pub(super) struct Key {
    pub(super) scancode: u8,
    pub(super) letter: char,
}
pub struct KeyTable;


impl KeyTable {
    pub fn get_letter(scancode: u8, state: &mut KeyState) -> char {
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
}