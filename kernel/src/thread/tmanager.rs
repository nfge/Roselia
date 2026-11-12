use core::sync::atomic::AtomicU64;

use alloc::{
    boxed::Box,
    collections::{BTreeMap, VecDeque},
    vec::Vec,
};
use kernel_api::thread::{Thread, ThreadContext, ThreadId};
use x86_64::{
    VirtAddr,
    registers::segmentation::{GS, Segment64},
};

use crate::{
    TASK_MANAGER, lock::interruptlock::InterruptSpinLock, memory::multi_allocator::free_pages,
    thread::scheduler::Scheduler,
};

pub struct TaskManager {
    threads: InterruptSpinLock<BTreeMap<ThreadId, Box<Thread>>>,
    sleepers: InterruptSpinLock<VecDeque<(u64, ThreadId)>>,
    zombies: InterruptSpinLock<Vec<ThreadId>>,
    id: InterruptSpinLock<AtomicU64>,
}

impl TaskManager {
    pub const fn new() -> Self {
        Self {
            threads: InterruptSpinLock::new(BTreeMap::new()),
            sleepers: InterruptSpinLock::new(VecDeque::new()),
            zombies: InterruptSpinLock::new(Vec::new()),
            id: InterruptSpinLock::new(AtomicU64::new(0)),
        }
    }
    pub fn push_thread(&self, thread: Box<Thread>) {
        self.threads.lock().insert(thread.id, thread);
    }
    pub fn push_sleeper(&self, time: u64, thread: Box<Thread>) {
        let mut sleepers = self.sleepers.lock();
        let i = sleepers.partition_point(|(w, _)| *w <= time);
        sleepers.insert(i, (time, thread.id));
    }
    pub fn wake_sleeper(&self, time: u64) -> Option<ThreadId> {
        loop {
            let id = {
                let mut sleepers = self.sleepers.lock();
                match sleepers.front() {
                    Some((w, _)) if *w <= time => sleepers.pop_front().unwrap().1,
                    _ => return None,
                }
            };
            if self
                .get_mut_by_tid(id, |t| t.state = kernel_api::thread::ThreadState::Ready)
                .is_some()
            {
                return Some(id);
            }
        }
    }
    pub fn push_zombie(&self, id: ThreadId) {
        self.zombies.lock().push(id);
    }
    pub fn get_by_tid<R>(&self, tid: ThreadId, f: impl FnOnce(&Thread) -> R) -> Option<R> {
        self.threads.lock().get(&tid).map(|t| f(t))
    }
    pub fn get_mut_by_tid<R>(&self, tid: ThreadId, f: impl FnOnce(&mut Thread) -> R) -> Option<R> {
        self.threads.lock().get_mut(&tid).map(|t| f(t))
    }
    pub fn new_id(&self) -> ThreadId {
        self.id
            .lock()
            .fetch_add(1, core::sync::atomic::Ordering::Relaxed)
    }
    pub fn release(&self, tid: ThreadId) {
        let Some(thread) = self.threads.lock().remove(&tid) else {
            return;
        };
        if thread.kernel_stack.len != 0 {
            free_pages(
                VirtAddr::new(thread.kernel_stack.ptr),
                thread.kernel_stack.len,
            );
        }
        drop(thread);
    }
}

pub fn reap() {
    loop {
        let zombie_id = TASK_MANAGER.zombies.lock().pop();
        let Some(tid) = zombie_id else { return };
        match TASK_MANAGER
            .get_by_tid(tid, |t| {
                if t.parent_id != None {
                    return t.reserved
                } else {
                    false
                }
            }) {
                Some(false) => TASK_MANAGER.release(tid),
                _ => {}
            }
       
    }
}
