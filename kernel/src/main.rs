#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]

extern crate alloc;

mod cpu;
mod func;
mod gop;
mod keyboard;
mod memory;
mod ramfs;
mod terminal;
mod timer;
mod uart;
use crate::{
    func::reset,
    gop::{color::Color, graphics::Graphics},
    ramfs::RamFs,
    terminal::Terminal,
    timer::sleep,
};
use alloc::boxed::Box;
use bootinfo::{BootInfo, reset::ResetFn, time::{GetTimeFn, KernelTime}, variable::{GetVar, SetVar}};
use core::{mem, panic::PanicInfo};
use uefi::{
    Error, mem::memory_map::MemoryMap, runtime::{Time, TimeCapabilities}
};

static mut FB_PTR: Option<*mut u32> = None;
static mut RESET_FN: Option<ResetFn> = None;
static mut TIME_FN: Option<GetTimeFn> = None;
static mut SET_VAR_FN: Option<SetVar> = None;
static mut GET_VAR_FN: Option<GetVar> = None;
static mut TERMINAL: *mut Terminal = core::ptr::null_mut();
static mut RAMFS: *mut RamFs = core::ptr::null_mut();

#[unsafe(no_mangle)]
pub extern "sysv64" fn kernel_main(boot_ptr: *const BootInfo) -> ! {
    let info = unsafe { &*boot_ptr };
    unsafe {
        FB_PTR = Some(info.gop.framebuffer_ptr as *mut u32);
        RESET_FN = Some(core::mem::transmute(info.reset));
        TIME_FN = Some(core::mem::transmute(info.time));
        SET_VAR_FN = Some(core::mem::transmute(info.set_var));
        GET_VAR_FN = Some(core::mem::transmute(info.get_var));
    };

    cpu::gdt::init();
    cpu::interrupts::init_idt();
    cpu::pic::disable_pic();
    cpu::apic::init_apic();
    cpu::apic::init_x2apic();
    cpu::apic::init_ioapic();
    cpu::apic::init_lapic();
    cpu::sse::init_sse();
    memory::init_heap(&info.memory_map);
    x86_64::instructions::interrupts::enable();
    
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
        if !TERMINAL.is_null() {
            (*TERMINAL).run();
        }
    }

    loop {
        x86_64::instructions::hlt();
    }
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    serial_println!("{}", _info);
    kprintln!("Kernel Panic: {}", _info);
    sleep(3000);
    reset(uefi::runtime::ResetType::COLD, uefi::Status::SUCCESS, None);
}
