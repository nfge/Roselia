use core::sync::atomic::{AtomicBool, Ordering};

use crate::keyboard::{KEYBOARD_BUFFER};
use kernel_api::keyboard::{keyevent::KeyEvent, keycode::{KeyCode,scancode_to_keycode}};


static SHIFT: AtomicBool = AtomicBool::new(false);
static CTRL: AtomicBool = AtomicBool::new(false);



pub fn handle_scancode(scancode: u8) {
    match scancode {
        0x2A | 0x36 => {
            SHIFT.store(true, Ordering::Relaxed);
            return;
        }
        0xAA | 0xB6 => {
            SHIFT.store(false, Ordering::Relaxed);
            return;
        }
        0x1D => {
            CTRL.store(true, Ordering::Relaxed);
            return;
        }
        0x9D => {
            CTRL.store(false, Ordering::Relaxed);
            return;
        }
        _ => {}
    }

    if scancode & 0x80 != 0 {
        return;
    }

    if let Some(code) = scancode_to_keycode(scancode) {
        KEYBOARD_BUFFER.lock().push(KeyEvent {
            code,
            shift: SHIFT.load(Ordering::Relaxed),
            ctrl: CTRL.load(Ordering::Relaxed),
        });
    }
}
