pub mod page_allocator;
mod bitmap;
use core::alloc::Layout;

use linked_list_allocator::LockedHeap;

use crate::memory::page_allocator::alloc_frames;

#[global_allocator]
static ALLOCATOR: LockedHeap = LockedHeap::empty();

pub fn init_heap() {
    let heap_size = 24 * 1024 * 1024;
    let pages = (heap_size + 4096 - 1) / 4096;
    let heap_start = alloc_frames(pages).unwrap();
    unsafe {ALLOCATOR.lock().init(heap_start.as_u64() as *mut u8, heap_size)};
}
pub fn get_heap_free() -> usize {
    ALLOCATOR.lock().free()
}
pub fn get_heap_used() -> usize {
    ALLOCATOR.lock().used()
}

pub fn kalloc(layout: Layout) -> Result<core::ptr::NonNull<u8>, ()> {
    ALLOCATOR.lock().allocate_first_fit(layout)
}
pub fn kfree(ptr: core::ptr::NonNull<u8>, layout: Layout) {
    unsafe {ALLOCATOR.lock().deallocate(ptr, layout)}
}