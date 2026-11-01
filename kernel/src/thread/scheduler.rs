use core::{ptr::null, sync::atomic::AtomicBool};

use alloc::{boxed::Box, collections::VecDeque};
use kernel_api::thread::{Thread, ThreadContext, ThreadId,ThreadState};
use x86_64::{PhysAddr, VirtAddr, registers::segmentation::{GS, Segment64}, structures::paging::{PageSize, PageTableFlags, Size4KiB}};

use crate::{TASK_MANAGER, lock::interruptlock::InterruptSpinLock, memory::multi_allocator::{alloc_pages, map}, thread::tmanager::TaskManager, timer::TICKS};

pub const TIME_MS: u64 = 8;

pub static SCHED_READY: AtomicBool = AtomicBool::new(false);

pub struct Scheduler {
    ready: InterruptSpinLock<VecDeque<ThreadId>>,
    current: Option<ThreadId>,
    previous: Option<ThreadId>,
    // pub idle: ThreadId,
    pub idle: Box<Thread>,
    pub remaining_time: u64
}

impl Scheduler {
    /// Create a new scheduler in current core
    pub fn new() {
        let idle_id = TASK_MANAGER.new_id();
        let rsp = alloc_pages(32).unwrap().as_u64();
        let ctx = ThreadContext::default();
        let base = Box::new(Self {
            ready: InterruptSpinLock::new(VecDeque::new()),
            current: None,
            previous: None,
            idle: Box::new(Thread { id: idle_id, context: ctx, kernel_stack: kernel_api::thread::KernelThreadStack { ptr: rsp, len: 32, top: rsp.checked_add(32 * Size4KiB::SIZE).unwrap()  }, state: kernel_api::thread::ThreadState::Ready }),
            remaining_time: TIME_MS
        });
        let ptr = Box::into_raw(base);
        unsafe {GS::write_base(VirtAddr::from_ptr(ptr))};
        SCHED_READY.store(true, core::sync::atomic::Ordering::Relaxed);
    }
    pub fn current() -> Option<&'static Self> {
        let ptr = GS::read_base().as_u64() as *const Scheduler;
        unsafe {ptr.as_ref()}
    }
    pub fn current_mut() -> Option<&'static mut Self> {
        let ptr = GS::read_base().as_u64() as *mut Scheduler;
        if ptr.is_null() {
            None
        } else {
            unsafe {Some(&mut *ptr)}
        }
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
    pub fn ready_is_empty(&self) -> bool {
        self.ready.lock().is_empty()
    }
    pub fn schedule(&mut self, ctx: *mut ThreadContext) -> *mut ThreadContext {
        self.remaining_time = TIME_MS;

        let task = &TASK_MANAGER;
        
        // if let Some(tid) = task.wake_sleeper(now) {
        //     self.ready.lock().push_back(tid);
        // }
        let current_state = task.get_by_tid(self.current.unwrap(), |f| return f.state);
        let next = match self.ready.lock().pop_front() {
            Some(t) => t,
            None => {
                if current_state == Some(kernel_api::thread::ThreadState::Running) {
                    return ctx
                } else {
                    self.idle.id
                }
            }
        };
        if next == self.current.unwrap() {
            return ctx;
        }
        task.get_mut_by_tid(self.current.unwrap(), |t| t.context = unsafe {*ctx});
        if current_state == Some(kernel_api::thread::ThreadState::Running) {
            task.get_mut_by_tid(self.current.unwrap(), |t| t.state = kernel_api::thread::ThreadState::Ready);
        }
        self.previous = self.current;
        self.current = Some(next);
        let new_ctx: *mut ThreadContext = task.get_mut_by_tid(next, |t| return &raw mut t.context).unwrap();
        self.switch_state();
        new_ctx
    }
    pub fn switch_state(&mut self) {
        let Some(prev) = self.previous.take() else {return};
        // if prev == self.idle {return};
        if prev == self.idle.id {return};
        match TASK_MANAGER.get_by_tid(prev, |t| return t.state) {
            Some(ThreadState::Running) => self.push_ready(prev),
            Some(ThreadState::Terminated) => TASK_MANAGER.push_zombie(prev),
            _ => {}
        }
    }
}
