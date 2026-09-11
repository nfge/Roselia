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
    memory::multi_allocator::{MultiAllocator, alloc_frame, map},
    module::{export::init_exports, load_module},
    ramfs::{RamFs, init_ramfs},
    terminal::Terminal,
    timer::sleep,
};

use alloc::{boxed::Box, vec::Vec};
use bootinfo::{
    BootInfo,
    reset::ResetFn,
    time::{GetTimeFn, OriginalGetTimeFn},
    variable::{GetVar, SetVar},
};
use core::{ffi::c_void, panic::PanicInfo};
use kernel_api::{
    acpi_tables::rsdp::Rsdp,
    module::{
        Module,
        raw::{RawModule, RawModules},
    },
};
use spin::mutex::Mutex;
use uefi::{boot::MemoryType, mem::memory_map::MemoryMap};
use utils::serial_println;
use x86_64::{
    PhysAddr, VirtAddr,
    registers::control::{Cr3, Cr3Flags},
    structures::paging::{OffsetPageTable, PageTable},
};

static mut FB_PTR: Option<*mut u32> = None;
static mut RESET_FN: Option<ResetFn> = None;
static mut TIME_FN: Option<OriginalGetTimeFn> = None;
static mut SET_VAR_FN: Option<SetVar> = None;
static mut GET_VAR_FN: Option<GetVar> = None;
static mut ACPI_TABLE: Option<*const c_void> = None;

static mut TERMINAL: *mut Terminal = core::ptr::null_mut();
static mut RAMFS: *mut RamFs = core::ptr::null_mut();
static mut MULTI_ALLOCATOR: Option<MultiAllocator<'static>> = None;
static mut MODULES: Option<Vec<Module>> = None;

static MAPPER: Mutex<Option<OffsetPageTable<'static>>> = Mutex::new(None);

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
    cpu::idt::init_idt();
    x86_64::instructions::interrupts::disable();
    cpu::pic::disable_pic();
    cpu::apic::init_apic();
    cpu::apic::init_x2apic();
    cpu::apic::init_ioapic();
    cpu::apic::init_lapic();
    x86_64::instructions::interrupts::enable();
    timer::calibrate();
    cpu::sse::init_sse_and_avx();
    unsafe {
        MULTI_ALLOCATOR = Some(MultiAllocator::new(&info.memory_map));
        if let Some(allocator) = &mut *core::ptr::addr_of_mut!(MULTI_ALLOCATOR) {
            allocator.init(
                info,
                &info.kernel_info,
                RawModules {
                    ptr: info.modules.ptr,
                    count: info.modules.count,
                },
            );
        }
    }
    x86_64::instructions::interrupts::without_interrupts(|| {
        let pml4_frame = alloc_frame().unwrap();
        let pml4_virt = VirtAddr::new(0 + pml4_frame.start_address().as_u64());
        let pml4: &mut PageTable = unsafe { &mut *pml4_virt.as_mut_ptr() };
        pml4.zero();
        let mapper = unsafe { OffsetPageTable::new(pml4, VirtAddr::new(0)) };
        *MAPPER.lock() = Some(mapper);
        let _ = map(pml4_frame.start_address(), pml4_frame.size() as usize);
        let _ = map(
            PhysAddr::new(info.kernel_info.stack_info.stack_ptr.as_ptr() as u64),
            info.kernel_info.stack_info.stack_pages,
        ).unwrap();
        let _ = map(PhysAddr::new(info as *const BootInfo as u64), core::mem::size_of::<BootInfo>()).unwrap();
        let _ = map(PhysAddr::new(info.kernel_info.start_address as u64), info.kernel_info.pages).unwrap();
        if let Some(allocator) = unsafe { &*core::ptr::addr_of_mut!(MULTI_ALLOCATOR) } {
            let _ = map(
                PhysAddr::new(allocator.bitmap.bitmap_start as u64),
                allocator.bitmap.bitmap_pages,
            )
            .unwrap();
        }
        if info.modules.count != 0 {
            let modules_bytes = info.modules.count * core::mem::size_of::<RawModule>();
            let modules_pages = modules_bytes.div_ceil(4096);
            let _ = map(PhysAddr::new(info.modules.ptr as u64), modules_pages).unwrap();
            for i in 0..info.modules.count {
                let module = unsafe { &*info.modules.ptr.add(i) };
                let raw_pages = (module.raw_len as usize).div_ceil(4096);
                let _ = map(PhysAddr::new(module.raw_ptr), raw_pages).unwrap();

                let image_pages = (module.len as usize).div_ceil(4096);
                let _ = map(PhysAddr::new(module.base), image_pages).unwrap();
            }
        }
        let _ = map(
            PhysAddr::new(info.gop.framebuffer_ptr as u64),
            info.gop.size.div_ceil(4096),
        )
        .unwrap();
        let _ = map(
            PhysAddr::new(info.memory_map.buffer().as_ptr() as u64),
            info.memory_map.len().div_ceil(4096),
        )
        .unwrap();
        for entry in info.memory_map.entries() {
            if entry.ty == MemoryType::RUNTIME_SERVICES_CODE
                || entry.ty == MemoryType::RUNTIME_SERVICES_DATA
            {
                let _ = map(PhysAddr::new(entry.phys_start), entry.page_count as usize).unwrap();
            }
        }

        for entry in info.memory_map.entries() {
            if entry.ty == MemoryType::ACPI_RECLAIM || entry.ty == MemoryType::ACPI_NON_VOLATILE {
                map(PhysAddr::new(entry.phys_start), entry.page_count as usize).unwrap();
            }
        }
        unsafe { Cr3::write(pml4_frame, Cr3::read().1) };
    });

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
