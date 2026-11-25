#![no_std]
#![no_main]

extern crate alloc;

use core::{alloc::{GlobalAlloc, Layout}, ptr::NonNull};

use kernel_api::{exports::export, module::{ABI_V1, MAGIC, ModuleInfo}};

pub struct ModuleAllocator;

unsafe impl GlobalAlloc for ModuleAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let mut ptr = unsafe {kernel_api::exports::kalloc(layout).unwrap()};
        unsafe {ptr.as_mut()}
    }
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        unsafe {kernel_api::exports::kfree(NonNull::new_unchecked(ptr), layout)};
    }
}

#[used]
#[unsafe(link_section = ".module_info")]
static MODULE_INFO: ModuleInfo = ModuleInfo {   
    name: *b"example\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
    mtype: kernel_api::module::ModuleType::Executable,
    abi_version: ABI_V1,
    module_version: 1.0,
    magic: MAGIC,
    flags: 0
};


// unsafe extern "Rust" {
//     fn kprintf(args: core::fmt::Arguments);
//     fn kprintfln(args: core::fmt::Arguments);
//     fn kalloc(layout:Layout) -> Result<core::ptr::NonNull<u8>, ()>;
//     fn kfree(ptr: NonNull<u8>, layout:Layout);
//     fn module_panic() -> !;
// }

#[unsafe(no_mangle)]
pub extern "C" fn module_init() -> i32 {
    unsafe {
        kernel_api::exports::kprintln!("This is example module!");
    };
    return 0
}

pub fn hello_world() {
    unsafe {kernel_api::exports::kprintln!("Hello World")};
}

export!("hello_world", hello_world);

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    unsafe {kernel_api::exports::module_panic()};
}


#[global_allocator]
pub static mut ALLOCATOR: ModuleAllocator = ModuleAllocator;
