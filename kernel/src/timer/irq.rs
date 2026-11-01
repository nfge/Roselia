use kernel_api::thread::ThreadContext;
use x86_64::{registers::segmentation::{GS, Segment64}, structures::idt::InterruptStackFrame};

use crate::{TASK_MANAGER, cpu, thread::scheduler::{SCHED_READY, Scheduler}};



// pub extern "C" fn timer_handler(ctx: *mut ThreadContext) -> *mut ThreadContext{
//     let now = super::TICKS.fetch_add(1, core::sync::atomic::Ordering::Relaxed) + 1;
//     cpu::apic::send_eoi();
//     if !SCHED_READY.load(core::sync::atomic::Ordering::Acquire) { return ctx; }
//     let scheduler = Scheduler::current_mut().unwrap();
//     while let Some(tid) = TASK_MANAGER.wake_sleeper(now) {
//         scheduler.push_ready(tid);
//     }
//     scheduler.remaining_time = scheduler.remaining_time.saturating_sub(1);
//     // let idle_run = scheduler.current_thread().unwrap() == scheduler.idle;
//     let idle_run = scheduler.current_thread().unwrap() == scheduler.idle.id;
//     if scheduler.remaining_time > 0 && !(idle_run && !scheduler.ready_is_empty()) {
//         return ctx;
//     }
//     scheduler.schedule(ctx)
// }
pub extern "x86-interrupt" fn timer_handler(_stack: InterruptStackFrame){
    super::TICKS.fetch_add(1, core::sync::atomic::Ordering::Relaxed);
    cpu::apic::send_eoi();
}