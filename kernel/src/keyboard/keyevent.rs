use crate::keyboard::keycode::KeyCode;

#[derive(Clone, Copy)]
pub struct KeyEvent {
    pub code: KeyCode,
    pub shift: bool,
    pub ctrl: bool
}