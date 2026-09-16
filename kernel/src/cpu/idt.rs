use core::hint::spin_loop;

use lazy_static::lazy_static;
use x86_64::{
    PhysAddr, VirtAddr,
    structures::{
        idt::{InterruptDescriptorTable, InterruptStackFrame, PageFaultErrorCode},
        paging::{Mapper, Page, PageSize, PageTableFlags, PhysFrame, Size4KiB},
    },
};

use crate::{MAPPER, MULTI_ALLOCATOR, keyboard, kprintln, log, log_fail, log_info, timer};
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

        unsafe {
            idt.general_protection_fault
                .set_handler_fn(gp_handler)
                .set_stack_index(1)
        };
        unsafe {
            idt.page_fault
                .set_handler_fn(pagefault_handler)
                .set_stack_index(2)
        };

        idt[33].set_handler_fn(keyboard::ps2::irq::keyboard_irq);
        idt[0xFF].set_handler_fn(spurious_handler);
        idt
    };
}

pub fn init_idt() {
    IDT.load();
}

extern "x86-interrupt" fn spurious_handler(_: InterruptStackFrame) {
    log_info!("Spurious");
}
// extern "x86-interrupt" fn default_handler(_:InterruptStackFrame){
//     loop {}
// }
extern "x86-interrupt" fn double_fault_handler(stack: InterruptStackFrame, _: u64) -> ! {
    serial_println!("Double fault");
    panic!("Double fault\n{:#?}", stack);
}
extern "x86-interrupt" fn invalid_opcode_handler(stack: InterruptStackFrame) {
    panic!("Invalid opcode\n{:#?}", stack)
}
extern "x86-interrupt" fn gp_handler(stack: InterruptStackFrame, code: u64) {
    serial_println!(
        "General Protection\nrip: {:#x}\ncs: {:#x}\nflags: {:#x}\nrsp: {:#x}\nss: {:#x}\ncode: {:#x}",
        stack.instruction_pointer.as_u64(),
        stack.code_segment.0,
        stack.cpu_flags.bits(),
        stack.stack_pointer.as_u64(),
        stack.stack_segment.0,
        code
    );
    loop {
        spin_loop();
    }
}
extern "x86-interrupt" fn debug_handler(stack: InterruptStackFrame) {
    serial_println!("[Debug] {:#?}", stack);
}
extern "x86-interrupt" fn breakpoint_handler(_stack: InterruptStackFrame) {
    serial_println!("Reached breakpoint at {:016x}\n", _stack.stack_pointer);
    log!("Reached breakpoint at {:016x}\n", _stack.stack_pointer);
}

extern "x86-interrupt" fn pagefault_handler(
    stack: InterruptStackFrame,
    err_code: PageFaultErrorCode,
) {
    use x86_64::registers::control::Cr2;

    panic!(
        "Page Fault\nrip: {:#x}\ncs: {:#x}\nflags: {:#x}\nrsp: {:#x}\nss: {:#x}\nerr_code: {:?}\nCr2: {:?}",
        stack.instruction_pointer.as_u64(),
        stack.code_segment.0,
        stack.cpu_flags.bits(),
        stack.stack_pointer.as_u64(),
        stack.stack_segment.0,
        err_code,
        Cr2::read()
    );
}
