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
    cpu::random::hardware_random,
    func::reset,
    gop::{color::Color, graphics::Graphics},
    memory::page_allocator::{PageAllocator, get_free_mem, get_total_memory, get_used_mem},
    module::{export::init_exports, load_module},
    ramfs::{RamFs, create_file, mkdir},
    terminal::Terminal,
    timer::sleep,
};

use acpi::get_table;
use alloc::{boxed::Box, format, vec::Vec};
use bootinfo::{
    BootInfo,
    reset::ResetFn,
    time::GetTimeFn,
    variable::{GetVar, SetVar},
};
use core::{
    ffi::c_void,
    panic::PanicInfo,
    ptr::{null, null_mut},
};
use kernel_api::{
    acpi_tables::mcfg::Mcfg,
    module::{Module, RawModules},
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
    cpu::sse::init_sse();

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
    memory::init_heap();

    unsafe {
        RAMFS = Box::into_raw(Box::new(RamFs::new()));
    }
    let _ = mkdir("/kernel").unwrap();
    let _ = mkdir("/sys").unwrap();
    let _ = mkdir("/dev").unwrap();
    let _ = create_file("/kernel/log", ramfs::data::NodeData::File(Vec::new())).unwrap();
    let _ = create_file(
        "/sys/memory",
        ramfs::data::NodeData::virtual_read(|| {
            let used = get_used_mem();
            let free = get_free_mem();
            let total = get_total_memory();
            format!("total: {}KB\nfree: {}KB\nused: {}KB\n", total, free, used).into_bytes()
        }),
    );
    let _ = create_file(
        "/kernel/info",
        ramfs::data::NodeData::virtual_read(|| {
            let version = env!("CARGO_PKG_VERSION");
            let git_commit = env!("GIT_COMMIT");
            let arch = if cfg!(target_arch = "x86_64") {
                "x86_64"
            } else {
                "Not Found"
            };
            format!(
                "Roselia Kernel {} ({})\nkernel.{}-{} {}\n",
                version, git_commit, version, git_commit, arch
            )
            .into_bytes()
        }),
    );
    let _ = mkdir("/dev/pci");
    let mcfg_ptr = unsafe { get_table::<Mcfg>(ACPI_TABLE.unwrap(), b"MCFG").unwrap() };
    let mcfg = unsafe { &*mcfg_ptr };
    let count = unsafe { mcfg.entry_count() };
    for i in 0..count {
        let entry = unsafe { &mcfg.entry(i) };
        let devices = unsafe { pci::enumerate::enumerate(entry) };
        for device in devices {
            let _ = create_file(
                format!(
                    "/dev/pci/{}:{}.{}",
                    device.bus, device.device, device.function
                )
                .as_str(),
                ramfs::data::NodeData::virtual_read(move || {
                    let (vendor_name, device_name) =
                        pci::check(device.header.vendor_id, device.header.device_id);
                    format!(
                        "{:04x} {}\n{:04x} {}\n\n",
                        device.header.vendor_id as u16,
                        vendor_name.unwrap_or("Not found in pci.ids"),
                        device.header.device_id as u16,
                        device_name.unwrap_or("Not found in pci.ids")
                    )
                    .into_bytes()
                }),
            );
        }
    }
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
