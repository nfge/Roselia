use crate::{cpu::apic, keyboard::KEYBOARD_BUFFER};

// extern "x86-interrupt" fn keyboard_irq(_stack_frame: x86_64::structures::idt::InterruptStackFrame) {
//     let keycode = unsafe { x86::io::inb(0x60) };
    
//     KEYBOARD_BUFFER.lock().push(keycode);
    
//     unsafe {apic::send_eoi()};
// }
