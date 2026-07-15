use lazy_static::lazy_static;
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame};

use crate::{keyboard, kprintln, timer};

lazy_static! {
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        
        unsafe {idt[32].set_handler_fn(timer::irq::timer_handler).set_stack_index(0)};
        idt[33].set_handler_fn(keyboard::irq::keyboard_irq);
        idt[0xFF].set_handler_fn(spurious_handler);
        idt
    };
}

pub fn init_idt() {
    IDT.load();
}

extern "x86-interrupt" fn spurious_handler(_: InterruptStackFrame) {
    kprintln!("Spurious");
}
// extern "x86-interrupt" fn default_handler(_:InterruptStackFrame){
//     loop {}
// }