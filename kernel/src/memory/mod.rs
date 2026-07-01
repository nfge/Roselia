mod frame_allocator;
mod stack;
use core::ptr::NonNull;

use linked_list_allocator::LockedHeap;
use uefi::{
    boot::{MemoryDescriptor, MemoryType},
    mem::memory_map::{MemoryMap, MemoryMapOwned},
};

#[global_allocator]
static ALLOCATOR: LockedHeap = LockedHeap::empty();

pub fn init_heap(mmap: &MemoryMapOwned) {
    let mut best_start: Option<*mut u8> = None;
    let mut best_size: usize = 0;

    for entry in mmap.entries() {
        if entry.ty == MemoryType::CONVENTIONAL {
            let size = entry.page_count as usize * 4096;

            if size > best_size {
                best_size = size;
                best_start = Some(entry.phys_start as *mut u8);
            }
        }
    }
    let heap_size = 500 * 1024 * 1024;
    let heap_start = best_start.expect("No usable memory");

    unsafe {ALLOCATOR.lock().init(heap_start, heap_size)};
}
pub fn get_free() -> usize {
    ALLOCATOR.lock().free()
}
pub fn get_used() -> usize {
    ALLOCATOR.lock().used()
}
pub fn get_size() -> usize {
    ALLOCATOR.lock().size()
}
pub fn alloc(layout: core::alloc::Layout) -> NonNull<u8> {
    let ptr = ALLOCATOR.lock().allocate_first_fit(layout).unwrap();
    ptr
}
pub fn dealloc(ptr: NonNull<u8>, layout: core::alloc::Layout) {
    unsafe {
        ALLOCATOR.lock().deallocate(ptr, layout);
    }
}
