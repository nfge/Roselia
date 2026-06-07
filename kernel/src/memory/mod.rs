use linked_list_allocator::LockedHeap;

use crate::HEAP_PTR;

#[global_allocator]
static ALLOCATOR:LockedHeap = LockedHeap::empty();


pub fn init_heap() {
    let heap_start = unsafe {HEAP_PTR.unwrap()};
    let heap_size = ((10 * 1024 * 1024 + 4095) / 4096) * 4096;
    unsafe {
        ALLOCATOR.lock().init(heap_start, heap_size);
    }
}
pub fn get_free() -> usize {
    ALLOCATOR.lock().free()
}
pub fn get_used() -> usize {
    ALLOCATOR.lock().used()
}