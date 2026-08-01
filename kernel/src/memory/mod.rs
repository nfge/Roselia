mod frame_allocator;
pub mod page_allocator;
mod bitmap;
use linked_list_allocator::LockedHeap;
use uefi::{
    boot::{MemoryType},
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
    let heap_size = 24 * 1024 * 1024;
    let heap_start = best_start.expect("No usable memory");

    unsafe {ALLOCATOR.lock().init(heap_start, heap_size)};
}
pub fn get_free() -> usize {
    ALLOCATOR.lock().free()
}
pub fn get_used() -> usize {
    ALLOCATOR.lock().used()
}