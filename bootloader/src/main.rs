#![no_std]
#![no_main]

mod init;
use elf_headers::{elf64ehdr::Elf64Ehdr,elf64phdr::Elf64Phdr};

use bootinfo::{BootInfo,reset::reset_fn,variable::{get_variable,set_variable},time::get_uefi_time};
// use uefi_raw::protocol::acpi::AcpiTableProtocol;



use core::{panic::PanicInfo, time::Duration};
use uefi::{
    CStr16,
    boot::{
        MemoryType, allocate_pages, exit_boot_services,
        get_image_file_system, image_handle,stall,
    },
    prelude::*,
    println,
    proto::{
        media::file::{self, File, FileAttribute, FileInfo, FileMode},
    },
    runtime::{ResetType},
};

use crate::{
    init::init_gop::init_gop,
};

const PT_LOAD: u32 = 1;

#[entry]
fn main() -> Status {
    uefi::helpers::init().unwrap();

    println!("Starting");

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
    let heap_pages = (10 * 1024 * 1024 + 4095) / 4096;
    let heap_ptr = allocate_pages(boot::AllocateType::AnyPages, MemoryType::LOADER_DATA, heap_pages).expect("Failed to allocate heap").as_ptr();
    let bootinfo = BootInfo {
        gop: framebuffer,
        reset: reset_fn,
        time: get_uefi_time,
        get_var: get_variable,
        set_var: set_variable,
        heap_ptr: heap_ptr
    };

    stall(Duration::from_secs(3));
    let _ = unsafe { exit_boot_services(None) };
    kernel_entry(&bootinfo as *const BootInfo);
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    println!("Paniced on: {}", _info);
    stall(Duration::from_secs(5));
    uefi::runtime::reset(ResetType::COLD, Status::LOAD_ERROR, None);
}
