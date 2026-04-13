use lazy_static::lazy_static;
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame};

use crate::{cpu::apic, keyboard, timer};

lazy_static! {
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        idt[0xFF].set_handler_fn(spurious_handler);
        idt[32].set_handler_fn(timer::irq::timer_handler);
        idt[33].set_handler_fn(keyboard::irq::keyboard_irq);
        idt
    };
}

pub fn init_idt() {
    IDT.load();
}

extern "x86-interrupt" fn spurious_handler(_: InterruptStackFrame) {
    apic::send_eoi();
}