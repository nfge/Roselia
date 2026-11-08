use core::mem::MaybeUninit;

use alloc::boxed::Box;
use kernel_api::thread::{KernelThreadStack, Thread, ThreadContext, ThreadId};
use x86_64::structures::paging::{PageSize, Size4KiB};

use crate::{TASK_MANAGER, memory::multi_allocator::alloc_pages, thread::scheduler::Scheduler};

pub mod scheduler;
pub mod tmanager;

pub fn spawn(f: fn() -> i32) -> ThreadId {
    x86_64::instructions::interrupts::without_interrupts(|| {
        let base = alloc_pages(32).unwrap().as_u64();
        let top = base.checked_add(32 * Size4KiB::SIZE).unwrap();
        let mut threadctx = ThreadContext::default();

        threadctx.rip = thread_trampoline as *const () as u64;
        threadctx.rdi = f as u64;

        let id = TASK_MANAGER.new_id();

        let thread = Thread::new(
            id,
            threadctx,
            KernelThreadStack::new(base, 32, top),
            kernel_api::thread::ThreadState::Ready,
        );
        TASK_MANAGER.push_thread(Box::new(thread));
        Scheduler::current_mut().unwrap().push_ready(id);

        id
    })
}

pub fn thread_trampoline(f: fn() -> i32) {
    let exit_code = f();

    thread_exit(exit_code);
}

pub fn thread_exit(exit_code: i32) {
    x86_64::instructions::interrupts::without_interrupts(|| {
        let schd = Scheduler::current_mut().unwrap();
        TASK_MANAGER
            .get_mut_by_tid(schd.current_thread().unwrap(), |t| {
                t.state = kernel_api::thread::ThreadState::Terminated;
                t.exit_code = MaybeUninit::new(exit_code);
            })
            .unwrap();
    })
}
