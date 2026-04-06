pub mod key_table;
pub mod key_state;
pub mod scancode_table;


use x86_64::instructions::port::Port;


use key_state::KeyState;
use key_table::KeyTable;

pub struct KeyBoard {
    pub key_state: KeyState,
}


impl KeyBoard {
    pub fn new() -> Self {
        Self {
            key_state: KeyState::new(),
        }
    }
    fn get_checkport() -> u8 {
        let mut scan_port: Port<u8> = Port::new(0x64);
        unsafe { scan_port.read() }
    }
    fn get_keycode() -> u8 {
        let mut scan_port: Port<u8> = Port::new(0x60);
        unsafe { scan_port.read() }
    }
    fn check_port() -> bool{
        let kport = Self::get_checkport();
        if (kport & 1) == 0 {
            return false;
        }
        return true;
    }
    pub fn get_key(&mut self) -> Option<char>{
        if Self::check_port() == true {
            let keycode = Self::get_keycode();
            self.key_state.set_state(keycode);
            let key = KeyTable::get_letter(keycode,&mut self.key_state);
            let released = (keycode & 0x80) != 0;
            if !released && key != '\0' {
                return Some(key);
            }   
        }
        None
    }
}