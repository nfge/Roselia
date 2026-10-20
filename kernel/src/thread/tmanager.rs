use alloc::{boxed::Box, collections::VecDeque, vec::Vec};
use kernel_api::thread::Thread;

use crate::lock::interruptlock::InterruptSpinLock;

pub struct TaskManager {
    ready: InterruptSpinLock<VecDeque<Box<Thread>>>,
    sleepers: InterruptSpinLock<VecDeque<(u64, Box<Thread>)>>,
    zombies: InterruptSpinLock<Vec<Box<Thread>>>,
}

impl TaskManager {
    // const fn new() -> Self {
    //     Self {
    //         ready: InterruptSpinLock::new(VecDeque::new()),
    //         sleepers: InterruptSpinLock::new(VecDeque::new()),
    //         zombies: InterruptSpinLock::new(Vec::new()),
    //     }
    // }
    // pub fn push_ready(&self, thread: Box<Thread>) {
    //     self.ready.lock().push_back(thread);
    // }
    // pub fn pop_ready(&self) -> Option<Box<Thread>> {
    //     self.ready.lock().pop_front()
    // }
    // pub fn push_sleeper(&self, time: u64, thread: Box<Thread>) {
    //     let mut sleepers = self.sleepers.lock();
    //     let i = sleepers.partition_point(|(w,_)| *w <= time);
    //     sleepers.insert(i, (time, thread));
    // }
    // pub fn wake_sleeper(&self, time: u64) {
    //     loop {
    //         let mut thread = {
    //             let mut sleepers = self.sleepers.lock();
    //             match sleepers.front() {
    //                 Some((w,_)) if *w <= time => sleepers.pop_front().unwrap().1,
    //                 _ => return
    //             }
    //         };
    //         thread.state = kernel_api::thread::ThreadState::Ready;
    //         self.push_ready(thread);
    //     }
    // }
    // pub fn push_zombie(&self, thread: Box<Thread>) {
    //     self.zombies.lock().push(thread);
    // }
    // pub fn reap(&self) {
    //     loop {
    //         let zombie = self.zombies.lock().pop();
    //         match zombie {
    //             Some(th) => drop(th),
    //             None => break
    //         }
    //     }
    // }
}


