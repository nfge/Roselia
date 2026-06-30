use core::ptr::NonNull;

use linked_list_allocator::LockedHeap;

use crate::HEAP_PTR;

#[global_allocator]
static ALLOCATOR:LockedHeap = LockedHeap::empty();


pub fn init_heap() {
    let heap_start = unsafe {HEAP_PTR.unwrap()};
    let heap_size = ((30 * 1024 * 1024 + 4095) / 4096) * 4096;
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
pub fn get_size() -> usize {
    ALLOCATOR.lock().size()
}
pub fn alloc(layout:core::alloc::Layout) -> NonNull<u8> {
    let ptr = ALLOCATOR.lock().allocate_first_fit(layout).unwrap();
    ptr
}
pub fn dealloc(ptr: NonNull<u8>, layout:core::alloc::Layout){
    unsafe {
        ALLOCATOR.lock().deallocate(ptr, layout);
    }
}