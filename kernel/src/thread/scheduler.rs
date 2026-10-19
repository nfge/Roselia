use alloc::boxed::Box;
use kernel_api::thread::Thread;

pub struct Scheduler {
    current: Option<Box<Thread>>,
    previous: Option<Box<Thread>>,
    idle: Box<Thread>
}