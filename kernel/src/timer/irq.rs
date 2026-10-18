use kernel_api::thread::ThreadContext;
use x86_64::structures::idt::InterruptStackFrame;

use crate::cpu;



// pub extern "C" fn timer_handler(_ctx: *mut ThreadContext){
//     super::TICKS.fetch_add(1, core::sync::atomic::Ordering::Relaxed);

//     cpu::apic::send_eoi();
// }
pub extern "x86-interrupt" fn timer_handler(_stack: InterruptStackFrame){
    super::TICKS.fetch_add(1, core::sync::atomic::Ordering::Relaxed);
    cpu::apic::send_eoi();
}