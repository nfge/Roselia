use kernel_api::thread::ThreadId;
use x86_64::structures::paging::{PageSize, Size4KiB};

use crate::memory::multi_allocator::alloc_pages;

pub mod tmanager;
pub mod scheduler;



pub fn spawn(f: fn() -> !) -> ThreadId {
    let base = alloc_pages(32).unwrap().as_u64();
    let top = base + 32 * Size4KiB::SIZE;
    todo!();
}