use x86_64::instructions::port::Port;

use crate::{keyboard::{key_state::KeyState, key_table::KeyTable}, vga::writer::Writer};
use core::fmt::Write;

pub struct KeyBoard {}


impl KeyBoard {

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
    pub fn get_key(writer: &mut Writer, state: &mut KeyState){
        if Self::check_port() == true {
            let keycode = Self::get_keycode();
            state.set_state(keycode);
            let key = KeyTable::get_letter(writer,keycode,state);
            let released = (keycode & 0x80) != 0;
            if !released && key != '\0' {
                let _ = write!(writer, "{}", key);
            }
            
        }
    }
}