pub mod irq;
// pub mod key_state;
// pub mod key_table;
mod ringbuffer;
// pub mod scancode_table;
mod special_keys;
pub mod keycode;
pub mod keyevent;

use x86_64::instructions::port::Port;

use crate::keyboard::{keycode::{KeyCode, scancode_to_keycode}, keyevent::KeyEvent};
pub static KEYBOARD_BUFFER: spin::Mutex<ringbuffer::RingBuffer<KeyEvent, 64>> =
    spin::Mutex::new(ringbuffer::RingBuffer::new());

pub struct KeyBoard {}

impl KeyBoard {
    pub fn new() -> Self {
        Self {}
    }
    fn get_checkport() -> u8 {
        let mut scan_port: Port<u8> = Port::new(0x64);
        unsafe { scan_port.read() }
    }
    fn get_keycode() -> u8 {
        let mut scan_port: Port<u8> = Port::new(0x60);
        unsafe { scan_port.read() }
    }
    fn check_port() -> bool {
        let kport = Self::get_checkport();
        if (kport & 1) == 0 {
            return false;
        }
        return true;
    }
    pub fn get_key(&mut self) -> Option<KeyEvent> {
        x86_64::instructions::interrupts::without_interrupts(|| {
            if let Some(keyevent) = KEYBOARD_BUFFER.lock().pop() {
                return Some(keyevent)
            }
            None
        })
    }
}
