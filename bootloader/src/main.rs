#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]

mod init;
use elf_headers::{elf64ehdr::Elf64Ehdr, elf64phdr::Elf64Phdr};

use bootinfo::{
    BootInfo,
    reset::reset_fn,
    time::get_uefi_time,
    variable::{get_variable, set_variable},
};

use core::{panic::PanicInfo, time::Duration, usize};
use uefi::{
    CStr16, boot::{
        EventType, MemoryType, TimerTrigger, Tpl, allocate_pages, create_event, exit_boot_services,
        get_handle_for_protocol, get_image_file_system, image_handle, memory_map,
        open_protocol_exclusive, set_timer, stall,
    }, mem::memory_map::MemoryMapOwned, prelude::*, println, proto::{
        console::text::{Input, Key, ScanCode},
        media::file::{self, File, FileAttribute, FileInfo, FileMode},
    }, runtime::{ResetType, VariableAttributes, VariableVendor, set_virtual_address_map}
};

use crate::init::init_gop::init_gop;

const PT_LOAD: u32 = 1;

#[entry]
fn main() -> Status {
    uefi::helpers::init().unwrap();

    println!("Press ESC to enter firmware setup (2 seconds timeout)");
    let timer = unsafe { create_event(EventType::TIMER, Tpl::APPLICATION, None, None).unwrap() };
    let input_handle = get_handle_for_protocol::<Input>().unwrap();
    let mut input = open_protocol_exclusive::<Input>(input_handle).unwrap();
    let _ = set_timer(&timer, TimerTrigger::Relative(20_000_000));
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
                stall(Duration::from_millis(300));
                uefi::runtime::reset(ResetType::WARM, Status::SUCCESS, None);
            }
            _ => {}
        },
    }

    let handle = image_handle();
    let mut filesys = get_image_file_system(handle).expect("Failed to load file system");

    let mut root = filesys.open_volume().expect("Failed to open volume");
    let kernel_name: &CStr16 = cstr16!("kernel.elf");
    let mut kernel = match root.open(&kernel_name, FileMode::Read, FileAttribute::empty()) {
        Ok(f) => match f.into_type() {
            Ok(file::FileType::Regular(file)) => file,
            _ => {
                println!("kernel.efi is not a regular file");
                return Status::UNSUPPORTED;
            }
        },
        Err(_) => {
            println!("Failed to open kernel.elf");
            return Status::NOT_FOUND;
        }
    };
    let mut buf = [0u8; 512];
    let info = kernel
        .get_info::<FileInfo>(&mut buf)
        .expect("Failed to get file info");

    let size = info.file_size() as usize;

    let pages = (size + 0xFFF) / 0x1000;

    let buffer_ptr = allocate_pages(boot::AllocateType::AnyPages, MemoryType::LOADER_DATA, pages)
        .expect("Failed to allocate pages")
        .as_ptr();
    let mut offset = 0;
    let chunk = 64 * 1024;
    while offset < size {
        let end = (offset + chunk).min(size);
        let slice =
            unsafe { core::slice::from_raw_parts_mut(buffer_ptr.add(offset), end - offset) };
        let read = kernel.read(slice).expect("Failed to read file");
        if read == 0 {
            break;
        }
        offset += read;
    }
    let ehdr = unsafe { &*(buffer_ptr as *const Elf64Ehdr) };
    if &ehdr.e_ident[0..4] != b"\x7FELF" {
        println!("Not ELF");
        return Status::LOAD_ERROR;
    }

    let mut last_allocated_start = 0usize;
    let mut last_allocated_end = 0usize;

    for i in 0..ehdr.e_phnum {
        let phdr = unsafe {
            &*(buffer_ptr.add(ehdr.e_phoff as usize + i as usize * ehdr.e_phentsize as usize)
                as *const Elf64Phdr)
        };

        if phdr.p_type != PT_LOAD {
            continue;
        }

        let pages = ((phdr.p_memsz + (phdr.p_vaddr % 0x1000) + 0xFFF) / 0x1000) as usize;
        let start_addr = (phdr.p_vaddr & !0xFFF) as usize;
        let end_addr = start_addr + (pages * 0x1000);

        if start_addr < last_allocated_start || start_addr >= last_allocated_end {
            let _ = boot::allocate_pages(
                boot::AllocateType::Address(start_addr as u64),
                boot::MemoryType::LOADER_DATA,
                pages,
            )
            .expect("Failed to reserve memory!");

            last_allocated_start = start_addr;
            last_allocated_end = end_addr;
        }

        let src = unsafe { buffer_ptr.add(phdr.p_offset as usize) };
        let dst = phdr.p_vaddr as *mut u8;
        unsafe {
            core::ptr::copy_nonoverlapping(src, dst, phdr.p_filesz as usize);

            if phdr.p_memsz > phdr.p_filesz {
                core::ptr::write_bytes(
                    dst.add(phdr.p_filesz as usize),
                    0,
                    (phdr.p_memsz - phdr.p_filesz) as usize,
                );
            }
        }
    }
    let entry = ehdr.e_entry;

    let kernel_entry: extern "sysv64" fn(boot_ptr: *const BootInfo) -> ! =
        unsafe { core::mem::transmute(entry as usize) };
    let framebuffer = init_gop();

    let mmap = unsafe { exit_boot_services(None) };
   
    let bootinfo = BootInfo {
        gop: framebuffer,
        reset: reset_fn as *const (),
        time: get_uefi_time as *const (),
        get_var: get_variable as *const (),
        set_var: set_variable as *const (),
        memory_map: mmap,
    };

    kernel_entry(&bootinfo as *const BootInfo);
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    println!("Paniced on: {}", _info);
    stall(Duration::from_secs(5));
    uefi::runtime::reset(ResetType::COLD, Status::LOAD_ERROR, None);
}
