#![no_std]
#![no_main]

mod init;
use bootinfo::{
    BootInfo,
    kernelinfo::KernelInfo,
    reset::reset_fn,
    time::get_uefi_time,
    variable::{get_variable, set_variable},
};
use kernel_api::module::{Module, Modules};

use core::{panic::PanicInfo, ptr::null, time::Duration, usize};
use uefi::{
    boot::{
        EventType, MemoryType, TimerTrigger, Tpl, allocate_pages, create_event, exit_boot_services, get_handle_for_protocol, get_image_file_system, image_handle, open_protocol_exclusive, set_timer, stall
    },
    prelude::*,
    println,
    proto::{
        console::text::{Input, Key, ScanCode},
        media::file::{File, FileAttribute},
    },
    runtime::{ResetType, VariableAttributes, VariableVendor},
    system::with_config_table,
    table::cfg::ConfigTableEntry,
};

use crate::init::{get_kernel, init_gop::init_gop, load_modules};

const PT_LOAD: u32 = 1;

#[entry]
fn main() -> Status {
    uefi::helpers::init().unwrap();

    println!("Press ESC to enter firmware setup (2 seconds timeout)");
    let timer = unsafe { create_event(EventType::TIMER, Tpl::APPLICATION, None, None).unwrap() };
    let input_handle = get_handle_for_protocol::<Input>().unwrap();
    let mut input = open_protocol_exclusive::<Input>(input_handle).unwrap();
    let _ = set_timer(&timer, TimerTrigger::Relative(Duration::from_secs(2)));
    let events = &mut [timer, input.wait_for_key_event().unwrap()];

    let index = boot::wait_for_event(events).discard_errdata().unwrap();

    match index {
        0 => {}
        1_usize.. => match input.read_key().unwrap() {
            Some(Key::Special(ScanCode::ESCAPE)) => {
                let mut current: [u8; 8] = [0; 8];
                let _ = uefi::runtime::get_variable(
                    uefi::cstr16!("OsIndications"),
                    &VariableVendor::GLOBAL_VARIABLE,
                    &mut current,
                );
                let mut val = u64::from_le_bytes(current);
                val |= 1;
                let new = val.to_le_bytes();
                let _ = uefi::runtime::set_variable(
                    uefi::cstr16!("OsIndications"),
                    &VariableVendor::GLOBAL_VARIABLE,
                    VariableAttributes::NON_VOLATILE
                        | VariableAttributes::RUNTIME_ACCESS
                        | VariableAttributes::BOOTSERVICE_ACCESS,
                    &new,
                );
                stall(Duration::from_millis(500));
                uefi::runtime::reset(ResetType::WARM, Status::SUCCESS, None);
            }
            _ => {}
        },
    }

    let handle = image_handle();
    let mut filesys = get_image_file_system(handle).expect("Failed to load file system");

    let (entry, kernel_start_addr, kernel_pages) = get_kernel(&mut filesys).unwrap();

    let (modules, modules_count) = load_modules(&mut filesys).unwrap();

    let kernel_entry: extern "sysv64" fn(boot_ptr: *const BootInfo) -> ! =
        unsafe { core::mem::transmute(entry as usize) };

    let framebuffer = init_gop();

    let mut acpi_ptr: *const core::ffi::c_void = null();
    with_config_table(|slice| {
        for i in slice {
            match i.guid {
                ConfigTableEntry::ACPI_GUID => acpi_ptr = i.address,
                ConfigTableEntry::ACPI2_GUID => acpi_ptr = i.address,
                _ => {}
            }
        }
    });
    let mmap = unsafe { exit_boot_services(None) };

    let bootinfo = BootInfo {
        kernel_info: KernelInfo {
            start_address: kernel_start_addr,
            pages: kernel_pages,
        },
        gop: framebuffer,
        reset: reset_fn as *const (),
        time: get_uefi_time as *const (),
        get_var: get_variable as *const (),
        set_var: set_variable as *const (),
        memory_map: mmap,
        acpi_table_ptr: acpi_ptr,
        modules: Modules {ptr: modules, count: modules_count}
    };

    kernel_entry(&bootinfo as *const BootInfo);
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    println!("Paniced on: {}", _info);
    stall(Duration::from_secs(5));
    uefi::runtime::reset(ResetType::COLD, Status::LOAD_ERROR, None);
}
