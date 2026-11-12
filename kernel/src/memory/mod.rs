mod bitmap;
pub mod multi_allocator;
pub mod pool_allocator;
use core::{
    alloc::{GlobalAlloc, Layout},
    ptr::NonNull,
};

use crate::{lock::interruptlock::InterruptSpinLock, memory::multi_allocator::alloc_pages};
use linked_list_allocator::LockedHeap;

struct KernelAllocator;

unsafe impl GlobalAlloc for KernelAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        kalloc(layout).map_or(core::ptr::null_mut(), |p| p.as_ptr())
    }
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        if let Some(p) = NonNull::new(ptr) {
            kfree(p, layout);
        }
    }
}

static ALLOCATOR: InterruptSpinLock<LockedHeap> = InterruptSpinLock::new(LockedHeap::empty());

#[global_allocator]
static KERNEL_ALLOCATOR: KernelAllocator = KernelAllocator;

pub fn init_heap() {
    let heap_size = 24 * 1024 * 1024;
    let pages = (heap_size + 4096 - 1) / 4096;
    let heap_start = alloc_pages(pages).unwrap();
    unsafe {
        ALLOCATOR
            .lock()
            .lock()
            .init(heap_start.as_u64() as *mut u8, heap_size)
    };
}
pub fn get_heap_free() -> usize {
    ALLOCATOR.lock().lock().free() / 1024
}
pub fn get_heap_used() -> usize {
    ALLOCATOR.lock().lock().used() / 1024
}

pub fn kalloc(layout: Layout) -> Result<core::ptr::NonNull<u8>, ()> {
    ALLOCATOR.lock().lock().allocate_first_fit(layout)
}
pub fn kfree(ptr: core::ptr::NonNull<u8>, layout: Layout) {
    unsafe { ALLOCATOR.lock().lock().deallocate(ptr, layout) }
}
