use crate::keyboard::KEYBOARD_BUFFER;

pub struct KeyState {
    pub shift: bool,
    pub ctrl: bool
}

impl KeyState {
    pub fn new() -> Self {
        Self {
            shift: false,
            ctrl: false
        }
    }
    pub fn set_state(&mut self, keycode: u8){
        match keycode {
            0x2A | 0x36 => self.shift = true,
            0xAA | 0xB6 => self.shift = false,
            0x1D => self.ctrl = true,
            0x9D => self.ctrl = false,
            _ => {}
        }
    }
    pub fn get_shift(&self) -> bool {
        self.shift
    }
    pub fn get_ctrl(&self) -> bool {
        self.ctrl
    } 
    pub fn update_state(&mut self) {
        x86_64::instructions::interrupts::without_interrupts(|| {
            if let Some(keycode) = KEYBOARD_BUFFER.lock().pop() {
                self.set_state(keycode);
            }
        })
    }
}