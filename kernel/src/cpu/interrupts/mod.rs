use lazy_static::lazy_static;
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame};

use crate::{keyboard, kprintln, log, timer};
use utils::serial_println;

lazy_static! {
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();

        unsafe {
            idt[32]
                .set_handler_fn(timer::irq::timer_handler)
                .set_stack_index(0)
        };
        unsafe {
            idt.double_fault
                .set_handler_fn(double_fault_handler)
                .set_stack_index(0)
        };
        idt.invalid_opcode.set_handler_fn(invalid_opcode_handler);
        idt.debug.set_handler_fn(debug_handler);
        idt.breakpoint.set_handler_fn(breakpoint_handler);
        idt.general_protection_fault.set_handler_fn(gp_handler);
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
extern "x86-interrupt" fn double_fault_handler(_: InterruptStackFrame, _: u64) -> ! {
    serial_println!("Double fault");
    loop {}
}
extern "x86-interrupt" fn invalid_opcode_handler(stack: InterruptStackFrame) {
    panic!("Invalid opcode\n{:#?}", stack)
}
extern "x86-interrupt" fn gp_handler(_stack: InterruptStackFrame, code: u64) {
    panic!("General Protection\n{:#?}", code);
}
extern "x86-interrupt" fn debug_handler(stack: InterruptStackFrame) {
    serial_println!("[Debug] {:#?}", stack);
}
extern "x86-interrupt" fn breakpoint_handler(_stack: InterruptStackFrame) {
    let secs = crate::timer::TICKS.load(core::sync::atomic::Ordering::Relaxed) / crate::timer::TICKS_PER_SEC.load(core::sync::atomic::Ordering::Relaxed);
    if secs == 0 {
        serial_println!("Reached breakpoint at {}ms", crate::timer::TICKS.load(core::sync::atomic::Ordering::Relaxed) * 1000 / crate::timer::TICKS_PER_SEC.load(core::sync::atomic::Ordering::Relaxed));
        log!("Reached breakpoint at {}ms\n", crate::timer::TICKS.load(core::sync::atomic::Ordering::Relaxed) * 1000 / crate::timer::TICKS_PER_SEC.load(core::sync::atomic::Ordering::Relaxed));
    } else {
        serial_println!("Reached breakpoint at {}s", secs);
        log!("Reached breakpoint at {}s\n", secs)
    }
    
}
