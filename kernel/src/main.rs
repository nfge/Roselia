#![no_std]
#![no_main]

mod vga;
mod keyboard;
use core::panic::PanicInfo;
use core::fmt::Write;

use crate::keyboard::key::KeyBoard;
use crate::vga::buffer::Buffer;
use crate::vga::writer::Writer;
use crate::vga::colors::{Color, ColorCode};

#[unsafe(no_mangle)]
pub extern "C" fn  _start() -> ! {
    
    // writer.write_string("Hello\n");
    // // for i in 0..81 {
    // //     writer.write_string("+");
    // // }
    // // writer.write_string("Privet Privet Privet Privet Privet Privet Privet Privet Privet Privet Privet Privet Privet Privet");
    // writer.write_string("Privet");
    let mut writer = Writer {
        row: 0,
        col: 0,
        color_code: ColorCode::new(Color::White, Color::Black),
        buffer: unsafe { &mut *(0xb8000 as *mut Buffer)}
    };
    let mut state = keyboard::key_state::KeyState {shift: false};
    loop {
        KeyBoard::get_key(&mut writer, &mut state);
    }
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    let mut krnlwriter = Writer {
        row: 0,
        col: 0,
        color_code: ColorCode::new(Color::Yellow, Color::Red),
        buffer: unsafe { &mut *(0xb8000 as *mut Buffer)}
    };
    krnlwriter.write_string("KERNEL PANIC!!!\n");
    let _ = write!(krnlwriter, "Error code: {}", _info);
    
    x86_64::instructions::interrupts::disable();
    loop {
        x86_64::instructions::hlt();
    }
}