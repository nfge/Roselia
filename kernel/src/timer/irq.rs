use x86_64::structures::idt::InterruptStackFrame;

use crate::cpu;

pub extern "x86-interrupt" fn timer_handler(_stack: InterruptStackFrame){
    // unsafe {x86::msr::wrmsr(0x80, 0)};
    cpu::apic::send_eoi();
}