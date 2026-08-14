use x86_64::instructions::port::Port;

use crate::{cpu::apic, keyboard::{keycode::handle_scancode}};

pub extern "x86-interrupt" fn keyboard_irq(_stack_frame: x86_64::structures::idt::InterruptStackFrame) {
    let mut keycode: Port<u8> = Port::new(0x60);
    handle_scancode(unsafe {keycode.read()});
    
    apic::send_eoi();
}

