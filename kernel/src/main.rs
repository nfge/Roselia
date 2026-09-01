#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]

extern crate alloc;

mod cpu;
mod func;
mod gop;
mod keyboard;
mod linker;
mod logger;
mod memory;
mod module;
mod ramfs;
mod terminal;
mod timer;

// mod uart;
use crate::{
    func::reset,
    gop::{color::Color, graphics::Graphics},
    memory::page_allocator::{PageAllocator, alloc_frame},
    module::{export::init_exports, load_module},
    ramfs::{RamFs, init_ramfs},
    terminal::Terminal,
    timer::sleep,
};

use alloc::{boxed::Box,vec::Vec};
use bootinfo::{
    BootInfo,
    reset::ResetFn,
    time::GetTimeFn,
    variable::{GetVar, SetVar},
};
use x86_64::{VirtAddr, structures::paging::{OffsetPageTable,PageTable}};
use core::{
    ffi::c_void, panic::PanicInfo,
};
use kernel_api::{
    module::{Module, raw::RawModules},
};
use utils::serial_println;

static mut FB_PTR: Option<*mut u32> = None;
static mut RESET_FN: Option<ResetFn> = None;
static mut TIME_FN: Option<GetTimeFn> = None;
static mut SET_VAR_FN: Option<SetVar> = None;
static mut GET_VAR_FN: Option<GetVar> = None;
static mut ACPI_TABLE: Option<*const c_void> = None;

static mut TERMINAL: *mut Terminal = core::ptr::null_mut();
static mut RAMFS: *mut RamFs = core::ptr::null_mut();
static mut PAGE_ALLOCATOR: Option<PageAllocator<'static>> = None;
static mut MODULES: Option<Vec<Module>> = None;

#[unsafe(no_mangle)]
pub extern "sysv64" fn kernel_main(boot_ptr: *const BootInfo) -> ! {
    let info = unsafe { &*boot_ptr };
    unsafe {
        FB_PTR = Some(info.gop.framebuffer_ptr as *mut u32);
        RESET_FN = Some(core::mem::transmute(info.reset));
        TIME_FN = Some(core::mem::transmute(info.time));
        SET_VAR_FN = Some(core::mem::transmute(info.set_var));
        GET_VAR_FN = Some(core::mem::transmute(info.get_var));
        ACPI_TABLE = Some(info.acpi_table_ptr);
    };
    cpu::gdt::init();
    cpu::interrupts::init_idt();
    cpu::pic::disable_pic();
    x86_64::instructions::interrupts::disable();
    cpu::apic::init_apic();
    cpu::apic::init_x2apic();
    cpu::apic::init_ioapic();
    cpu::apic::init_lapic();
    x86_64::instructions::interrupts::enable();
    timer::calibrate();
    cpu::sse::init_sse_and_avx();

    unsafe {
        PAGE_ALLOCATOR = Some(PageAllocator::new(&info.memory_map));
        if let Some(allocator) = &mut *core::ptr::addr_of_mut!(PAGE_ALLOCATOR) {
            allocator.init(
                info.kernel_info.start_address,
                info.kernel_info.pages,
                RawModules {
                    ptr: info.modules.ptr,
                    count: info.modules.count,
                },
            );
        }
    }
    let pml4_frame = alloc_frame().unwrap();
    let pml4_virt = VirtAddr::new(0 + pml4_frame.start_address().as_u64());
    let pml4: &mut PageTable = unsafe {&mut *pml4_virt.as_mut_ptr()};
    pml4.zero();
    let mut mapper = unsafe { OffsetPageTable::new(pml4, VirtAddr::new(0))};
    memory::init_heap();

    unsafe {
        RAMFS = Box::into_raw(Box::new(RamFs::new()));
    }
    init_ramfs();
    init_exports();
    unsafe { MODULES = Some(Vec::new()) }
    if info.modules.count != 0 {
        for i in 0..info.modules.count {
            let rawmodule = unsafe { &*info.modules.ptr.add(i) };
            let module = unsafe { load_module(&rawmodule).unwrap() };
            unsafe {
                if let Some(modules) = &mut *core::ptr::addr_of_mut!(MODULES) {
                    modules.push(module);
                }
            }
        }
    }
    unsafe {
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
    unsafe { reset() };
}
