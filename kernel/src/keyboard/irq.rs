use x86_64::instructions::port::Port;

use crate::{cpu::apic, keyboard::KEYBOARD_BUFFER};

pub extern "x86-interrupt" fn keyboard_irq(_stack_frame: x86_64::structures::idt::InterruptStackFrame) {
    let mut keycode: Port<u8> = Port::new(0x60);
    
    KEYBOARD_BUFFER.lock().push(unsafe {keycode.read()});
    
    apic::send_eoi();
}

