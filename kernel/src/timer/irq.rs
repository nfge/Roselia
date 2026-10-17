use kernel_api::thread::ThreadContext;

use crate::cpu;



pub extern "C" fn timer_handler(_ctx: *mut ThreadContext){
    super::TICKS.fetch_add(1, core::sync::atomic::Ordering::Relaxed);

    cpu::apic::send_eoi();
}