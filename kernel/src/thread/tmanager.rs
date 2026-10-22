use core::sync::atomic::{AtomicU64};

use alloc::{boxed::Box, collections::{BTreeMap, VecDeque}, vec::Vec};
use kernel_api::thread::{Thread, ThreadId};

use crate::lock::interruptlock::InterruptSpinLock;

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
            id: InterruptSpinLock::new(AtomicU64::new(0))
        }
    }
    pub fn push_sleeper(&self, time: u64, thread: Box<Thread>) {
        let mut sleepers = self.sleepers.lock();
        let i = sleepers.partition_point(|(w,_)| *w <= time);
        sleepers.insert(i, (time, thread.id));
    }
    pub fn wake_sleeper(&self, time: u64) {
        loop {
            let mut id = {
                let mut sleepers = self.sleepers.lock();
                match sleepers.front() {
                    Some((w,_)) if *w <= time => sleepers.pop_front().unwrap().1,
                    _ => return
                }
            };
            if self.get_mut_by_tid(id, |t| t.state = kernel_api::thread::ThreadState::Ready).is_some() {
                todo!()
            }
        }
    }
    pub fn push_zombie(&self, thread: Box<Thread>) {
        self.zombies.lock().push(thread.id);
    }
    pub fn reap(&self) {
        loop {
            let zombie = self.zombies.lock().pop();
            match zombie {
                Some(th) => drop(th),
                None => break
            }
        }
    }
    pub fn get_by_tid<R>(&self, tid: ThreadId, f:impl FnOnce(&Thread) -> R) -> Option<R> {
        self.threads.lock().get(&tid).map(|t| f(t))
    }
    pub fn get_mut_by_tid<R>(&self, tid: ThreadId, f:impl FnOnce(&mut Thread) -> R) -> Option<R> {
        self.threads.lock().get_mut(&tid).map(|t| f(t))
    }
    pub fn new_id(&self) -> ThreadId {
        self.id.lock().fetch_add(1, core::sync::atomic::Ordering::Relaxed)
    }
}


