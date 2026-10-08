use kernel_api::keyboard::keyevent::KeyEvent;
use core::fmt::Write;
use crate::TERMINAL;

pub extern "Rust" fn kprint(s: &str) {
    unsafe {
        if !TERMINAL.is_null() {
            let term = TERMINAL;
            let _ = (*term).print_fmt(format_args!("{s}"));
        }
    }
}
pub extern "Rust" fn kprintln(s: &str) {
    unsafe {
        if !TERMINAL.is_null() {
            let term = TERMINAL;
            let _ = (*term).print_fmt(format_args!("{s}"));
        }
    }
}

pub extern "Rust" fn kprintf(arg: core::fmt::Arguments) {
    unsafe {
        if !TERMINAL.is_null() {
            let term = TERMINAL;
            let _ = (*term).print_fmt(arg);
        }
    }
}
pub extern "Rust" fn kprintfln(arg: core::fmt::Arguments) {
    kprintf(arg);
    kprintf(format_args!("\n"));
}

pub extern "Rust" fn get_key() -> Option<KeyEvent> {
    if unsafe {!TERMINAL.is_null()} {
        let term = unsafe {TERMINAL};
        unsafe {
            return (*term).keyboard.get_key()
        }
    }
    None
}
pub extern "Rust" fn set_cursor_cell(cell_x: Option<usize>, cell_y: Option<usize>) {
    if unsafe {!TERMINAL.is_null()} {
        let term = unsafe {TERMINAL};
        unsafe {
            (*term).set_cursor_cell(cell_x, cell_y);
        }
    }
}

// #[unsafe(no_mangle)]
// pub extern "C" fn kernel_printchar(c: u8) {
//     if unsafe {!TERMINAL.is_null()} {
//         unsafe {(*TERMINAL).print_char(c as char);}
//     }
// }
