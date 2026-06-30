#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]

extern crate alloc;

mod cpu;
mod gop;
mod keyboard;
mod memory;
mod terminal;
mod timer;
mod uart;
mod ramfs;
use crate::{
    gop::{color::Color, graphics::Graphics},
    terminal::Terminal,
    timer::sleep,
    ramfs::RamFs
};
use alloc::{boxed::Box, format};
use bootinfo::{BootInfo};
use core::panic::PanicInfo;
use uefi::{
    Error, mem::memory_map::MemoryMap, runtime::{Time, TimeCapabilities}
};


static mut FB_PTR: Option<*mut u32> = None;
static mut HEAP_PTR: Option<*mut u8> = None;
static mut RESET_FN: Option<
    fn(reset_type: uefi::runtime::ResetType, status: uefi::Status, data: Option<&[u8]>) -> !,
> = None;
static mut TIME_FN: Option<fn() -> Result<(Time, TimeCapabilities), Error<()>>> = None;
static mut SET_VAR_FN: Option<
    fn(
        name: &uefi::CStr16,
        vendor: &uefi::runtime::VariableVendor,
        attributes: uefi::runtime::VariableAttributes,
        data: &[u8],
    ) -> Result<(), uefi::Error>,
> = None;
static mut GET_VAR_FN: Option<
    for<'buf> fn(
        name: &uefi::CStr16,
        vendor: &uefi::runtime::VariableVendor,
        buf: &'buf mut [u8],
    ) -> Result<
        (&'buf [u8], uefi::runtime::VariableAttributes),
        uefi::Error<Option<usize>>,
    >,
> = None;
static mut TERMINAL: *mut Terminal = core::ptr::null_mut();
static mut RAMFS: *mut RamFs = core::ptr::null_mut();

#[unsafe(no_mangle)]
pub fn kernel_main(boot_ptr: *const BootInfo) -> ! {
    let info = unsafe { &*boot_ptr };
    unsafe {
        FB_PTR = Some(info.gop.framebuffer_ptr as *mut u32);
        HEAP_PTR = Some(info.heap_ptr);
        RESET_FN = Some(info.reset);
        TIME_FN = Some(info.time);
        SET_VAR_FN = Some(info.set_var);
        GET_VAR_FN = Some(info.get_var)
    };
    cpu::interrupts::init_idt();
    cpu::pic::disable_pic();
    cpu::apic::init_apic();
    cpu::sse::init_sse();
    memory::init_heap();
    unsafe {
        RAMFS = Box::into_raw(Box::new(RamFs::new()));
        TERMINAL = Box::into_raw(Box::new(Terminal::new(
            Graphics::new(info.gop.framebuffer_ptr, info.gop.mode_info),
            0,
            0,
            1,
            Color::White,
        )));
    }
    
    unsafe {
        if !TERMINAL.is_null() {
            (*TERMINAL).flush_screen();
        }
    }
    for entry in info.memory_map.entries() {
        if entry.ty == uefi::mem::memory_map::MemoryType::CONVENTIONAL {
            kprint!("Physical addr: 0x{} ", entry.phys_start);
            kprintln!("Page Count: {}", entry.page_count)
        }
    }
    
    loop {
        x86_64::instructions::hlt();
    }
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    kprintln!("Kernel Panic: {}", _info);
    kprintln!("\nPress Enter to reset");
    unsafe { 
        if !TERMINAL.is_null() {
            loop { if (*TERMINAL).wait_for_key('\n') == true {break}};
            
        }
    }
    kprintln!("Reseting...");
    sleep(100);
    unsafe { RESET_FN.unwrap()(uefi::runtime::ResetType::COLD, uefi::Status::SUCCESS, None) };
}
