#![no_std]
#![no_main]

use kernel_api::module::ModuleInfo;

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
    fn kprint(s: &str);
}

#[unsafe(no_mangle)]
pub extern "C" fn module_init() {
    unsafe {kprint("Hello World")};
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}