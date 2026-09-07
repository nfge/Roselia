mod ringbuffer;
pub mod ps2;
pub mod usb;

use kernel_api::keyboard::{keyevent::KeyEvent};

pub static KEYBOARD_BUFFER: spin::Mutex<ringbuffer::RingBuffer<KeyEvent, 64>> =
    spin::Mutex::new(ringbuffer::RingBuffer::new());

pub struct KeyBoard {}

impl KeyBoard {
    pub fn new() -> Self {
        Self {}
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
