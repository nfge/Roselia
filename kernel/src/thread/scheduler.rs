use alloc::{boxed::Box, collections::VecDeque};
use kernel_api::thread::{Thread, ThreadContext, ThreadId};
use x86_64::{VirtAddr, registers::segmentation::{GS, Segment64}, structures::paging::{PageSize, Size4KiB}};

use crate::{TASK_MANAGER, lock::interruptlock::InterruptSpinLock, memory::multi_allocator::alloc_pages, thread::tmanager::TaskManager};

pub struct Scheduler {
    ready: InterruptSpinLock<VecDeque<ThreadId>>,
    current: Option<ThreadId>,
    previous: Option<ThreadId>,
    idle: Box<Thread>
}

impl Scheduler {
    pub fn new() -> Self {
        let idle_id = unsafe {&*core::ptr::addr_of_mut!(TASK_MANAGER)}.as_ref().unwrap().new_id();
        let rsp = alloc_pages(32).unwrap().as_u64();
        let ctx = unsafe {
            core::ptr::read(rsp as *const ThreadContext)
        };
        Self {
            ready: InterruptSpinLock::new(VecDeque::new()),
            current: None,
            previous: None,
            idle: Box::new(Thread { id: idle_id, context: ctx, kernel_stack: kernel_api::thread::KernelThreadStack { ptr: rsp, len: 32, top: rsp.checked_add(32 * Size4KiB::SIZE).unwrap()  }, state: kernel_api::thread::ThreadState::Ready })
        }
    }
    pub fn push_ready(&self, tid: ThreadId) {
        self.ready.lock().push_back(tid);
    }
    pub fn pop_ready(&self) -> Option<ThreadId> {
        self.ready.lock().pop_back()
    }
}