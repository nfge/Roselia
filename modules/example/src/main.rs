#![no_std]
#![no_main]

extern crate alloc;

use core::{alloc::{GlobalAlloc, Layout}, ptr::NonNull};

use kernel_api::module::{ModuleInfo};

pub struct ModuleAllocator;

unsafe impl GlobalAlloc for ModuleAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let mut ptr = unsafe {kalloc(layout).unwrap()};
        unsafe {ptr.as_mut()}
    }
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        unsafe {kfree(NonNull::new_unchecked(ptr), layout)};
    }
}

#[used]
#[unsafe(link_section = ".module_info")]
static MODULE_INFO: ModuleInfo = ModuleInfo {   
    name: *b"example\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
    abi_version: 1,
    module_version: 1,
    magic: 0x524F_5345_4C49_4100,
    flags: 0
};


unsafe extern "Rust" {
    fn kprintf(args: core::fmt::Arguments);
    fn kprintfln(args: core::fmt::Arguments);
    fn kalloc(layout:Layout) -> Result<core::ptr::NonNull<u8>, ()>;
    fn kfree(ptr: NonNull<u8>, layout:Layout);
    fn module_panic() -> !;
}

#[unsafe(no_mangle)]
pub extern "C" fn module_init() -> i32 {
    unsafe {
        kprintf(format_args!("This is example module!\n"));
    };
    return 0
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    unsafe {module_panic()};
}


#[global_allocator]
pub static mut ALLOCATOR: ModuleAllocator = ModuleAllocator;
