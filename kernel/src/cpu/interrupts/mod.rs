use lazy_static::lazy_static;
use x86_64::structures::idt::InterruptDescriptorTable;

use crate::keyboard;

lazy_static! {
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        idt[33].set_handler_fn(keyboard::irq::keyboard_irq);
        idt
    };
}

pub fn init_idt() {
    IDT.load();
}
