// use core::ptr::addr_of_mut;

// use crate::ALLOCATOR;

// const HEAP_SIZE: usize = 1024 * 1024;
// static mut HEAP: [u8; 1024 * 1024] = [0; 1024 * 1024];

// pub fn init_alloc(){
//     unsafe {
//         let heap_start = addr_of_mut!(HEAP) as *mut u8;
//         ALLOCATOR.lock().init(heap_start, HEAP_SIZE);
//     }
// }