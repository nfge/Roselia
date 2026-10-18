use alloc::{boxed::Box, collections::VecDeque, vec::Vec};
use kernel_api::thread::Thread;

pub struct TaskManager {
    threads: Vec<Thread>,
    ready: VecDeque<Box<Thread>>,
    // ...
}