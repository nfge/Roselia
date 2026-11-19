use core::sync::atomic::AtomicU64;

use alloc::{
    boxed::Box,
    collections::{BTreeMap, VecDeque},
    vec::Vec,
};
use kernel_api::thread::{Thread, ThreadContext, ThreadId, ThreadState};
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
    pub fn push_sleeper(&self, time: u64, id: ThreadId) {
        let mut sleepers = self.sleepers.lock();
        let i = sleepers.partition_point(|(w, _)| *w <= time);
        sleepers.insert(i, (time, id));
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
            if self.get_mut_by_tid(id, |t| {
                if t.state == ThreadState::Sleeping {
                    t.state = ThreadState::Ready;
                    true
                } else {
                    false
                }
            }) == Some(true)
            {
                return Some(id);
            }
        }
    }
    pub fn push_zombie(&self, id: ThreadId) {
        self.zombies.lock().push(id);
    }
    fn take_zombie(&self) -> Option<ThreadId> {
        let mut zombies = self.zombies.lock();
        let pos = zombies.iter().position(|&tid| self.is_zombie(tid))?;
        Some(zombies.swap_remove(pos))
    }
    fn is_zombie(&self, tid: ThreadId) -> bool {
        match self.get_by_tid(tid, |t| t.parent_id) {
            None => true,
            Some(None) => true,
            Some(Some(p)) => self
                .get_by_tid(p, |t| t.state == ThreadState::Terminated)
                .unwrap_or(true),
        }
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
    while let Some(tid) = TASK_MANAGER.take_zombie() {
        TASK_MANAGER.release(tid);
    }
}
