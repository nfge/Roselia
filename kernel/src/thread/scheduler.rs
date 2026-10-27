use core::ptr::null;

use alloc::{boxed::Box, collections::VecDeque};
use kernel_api::thread::{Thread, ThreadContext, ThreadId};
use x86_64::{PhysAddr, VirtAddr, registers::segmentation::{GS, Segment64}, structures::paging::{PageSize, PageTableFlags, Size4KiB}};

use crate::{TASK_MANAGER, lock::interruptlock::InterruptSpinLock, memory::multi_allocator::{alloc_pages, map}, thread::tmanager::TaskManager};

pub struct Scheduler {
    ready: InterruptSpinLock<VecDeque<ThreadId>>,
    current: Option<ThreadId>,
    previous: Option<ThreadId>,
    idle: Box<Thread>
}

impl Scheduler {
    /// Create a new scheduler in current core
    pub fn new() {
        let idle_id = unsafe {&*core::ptr::addr_of_mut!(TASK_MANAGER)}.as_ref().unwrap().new_id();
        let rsp = alloc_pages(32).unwrap().as_u64();
        let ctx = ThreadContext::default();
        let base = Box::new(Self {
            ready: InterruptSpinLock::new(VecDeque::new()),
            current: None,
            previous: None,
            idle: Box::new(Thread { id: idle_id, context: ctx, kernel_stack: kernel_api::thread::KernelThreadStack { ptr: rsp, len: 32, top: rsp.checked_add(32 * Size4KiB::SIZE).unwrap()  }, state: kernel_api::thread::ThreadState::Ready })
        });
        let ptr = Box::into_raw(base);
        unsafe {GS::write_base(VirtAddr::from_ptr(ptr))};
    }
    pub fn current() -> Option<&'static Self> {
        let ptr = GS::read_base().as_u64() as *const Scheduler;
        unsafe {ptr.as_ref()}
    }
    pub fn current_thread(&self) -> Option<ThreadId> {
        self.current
    }
    pub fn push_ready(&self, tid: ThreadId) {
        self.ready.lock().push_back(tid);
    }
    pub fn pop_ready(&self) -> Option<ThreadId> {
        self.ready.lock().pop_back()
    }
}