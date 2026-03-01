pub(crate) struct KeyState {
    pub shift: bool,
}

impl KeyState {
    pub fn set_state(&mut self, keycode: u8){
        match keycode {
            0x2A | 0x36 => self.shift = true,
            0xAA | 0xB6 => self.shift = false,
            _ => {}
        }
    }
}