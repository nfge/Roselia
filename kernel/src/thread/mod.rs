use core::mem::MaybeUninit;

use alloc::boxed::Box;
use kernel_api::thread::{KernelThreadStack, Thread, ThreadContext, ThreadId, ThreadState};
use x86_64::{
    registers::segmentation::{CS, SS, Segment},
    structures::paging::{PageSize, Size4KiB},
};

use crate::{TASK_MANAGER, memory::multi_allocator::alloc_pages, thread::scheduler::Scheduler};

pub mod scheduler;
pub mod tmanager;

fn new_ctx(entry: u64, arg: u64, stack_top: u64) -> ThreadContext {
    let mut tc = ThreadContext::default();
    tc.rip = entry;
    tc.rdi = arg;
    tc.cs = CS::get_reg().0 as u64;
    tc.ss = SS::get_reg().0 as u64;
    tc.rflags = 0x202;
    tc.rsp = stack_top - 8;
    tc
}

pub fn spawn<F>(f: F) -> ThreadId
where
    F: FnOnce() -> i32 + Send + 'static {
    
    crate::thread::tmanager::reap();
    let arg = Box::into_raw(Box::new(f)) as u64;
    x86_64::instructions::interrupts::without_interrupts(|| {
        let base = alloc_pages(32).unwrap().as_u64();
        let top = base.checked_add(32 * Size4KiB::SIZE).unwrap();
        let threadctx = new_ctx(thread_trampoline::<F> as *const () as u64, arg, top);

        let id = TASK_MANAGER.new_id();
        let parent = Scheduler::current().and_then(|s| s.current_thread());
        let mut thread = Thread::new(
            id,
            threadctx,
            KernelThreadStack::new(base, 32, top),
            kernel_api::thread::ThreadState::Ready,
        );
        thread.parent_id = parent;
        TASK_MANAGER.push_thread(Box::new(thread));
        Scheduler::current_mut().unwrap().push_ready(id);

        id
    })
}

extern "C" fn thread_trampoline<F: FnOnce() -> i32>(arg: *mut F) {
    let exit_code = {
        let f = unsafe { Box::from_raw(arg) };
        (*f)()
    };

    thread_exit(exit_code);
}

pub fn thread_exit(exit_code: i32) {
    x86_64::instructions::interrupts::without_interrupts(|| {
        let schd = Scheduler::current_mut().unwrap();
        TASK_MANAGER
            .get_mut_by_tid(schd.current_thread().unwrap(), |t| {
                t.state = kernel_api::thread::ThreadState::Terminated;
                t.exit_code = Some(exit_code);
            })
            .unwrap();
        schd.remaining_time = 0;
    });
    loop {
        x86_64::instructions::interrupts::enable_and_hlt();
    }
}

pub fn wait(tid: ThreadId) -> Option<i32> {
    let current = Scheduler::current().unwrap().current_thread().unwrap();
    loop {
        let res = TASK_MANAGER.get_mut_by_tid(tid, |t| {
            if t.parent_id != Some(current) {
                return Err(())
            }
            if t.state != ThreadState::Terminated {
                return Ok(None)
            }
            t.parent_id = None;
            return Ok(t.exit_code.take())
        });

        match res {
            None | Some(Err(())) => return None,
            Some(Ok(Some(code))) => {
                TASK_MANAGER.get_mut_by_tid(tid, |t| t.reserved = false);
                return Some(code)
            }
            Some(Ok(None)) => x86_64::instructions::hlt(), 
        }
        
    }
}
