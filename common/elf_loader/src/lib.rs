#![no_std]
#![no_main]

use kernel_api::{elf::{elf64ehdr::Elf64Ehdr, elf64phdr::Elf64Phdr, elf64phdr::PT_LOAD}, module::RawModule};
use uefi::{
    CStr16, Status,
    boot::{MemoryType, ScopedProtocol, allocate_pages},
    println,
    proto::media::{
        file::{Directory, File, FileAttribute, FileInfo, FileType},
        fs::SimpleFileSystem,
    },
};

pub fn load_elf(
    modules_dir: &mut Directory,
    name: &CStr16,
) -> Result<RawModule, uefi::Status> {
    let mut file = match modules_dir.open(
        &name,
        uefi::proto::media::file::FileMode::Read,
        FileAttribute::empty(),
    ) {
        Ok(f) => match f.into_type() {
            Ok(FileType::Regular(file)) => file,
            _ => {
                println!("{name} is not a regular file");
                return Err(Status::UNSUPPORTED);
            }
        },
        Err(_) => {
            println!("Failed to open {name}");
            return Err(Status::NOT_FOUND);
        }
    };
    let mut buf = [0u8; 512];
    let info = file
        .get_info::<FileInfo>(&mut buf)
        .expect("Failed to get file info");

    let size = info.file_size() as usize;

    let pages = (size + 0xFFF) / 0x1000;

    let buffer_ptr = allocate_pages(uefi::boot::AllocateType::AnyPages, MemoryType::LOADER_DATA, pages)
        .expect("Failed to allocate pages")
        .as_ptr();
    let mut offset = 0;
    let chunk = 64 * 1024;

    while offset < size {
        let end = (offset + chunk).min(size);
        let slice =
            unsafe { core::slice::from_raw_parts_mut(buffer_ptr.add(offset), end - offset) };
        let read = file.read(slice).expect("Failed to read file");
        if read == 0 {
            break;
        }
        offset += read;
    }

    let ehdr = unsafe { &*(buffer_ptr as *const Elf64Ehdr) };
    if &ehdr.e_ident[0..4] != b"\x7FELF" {
        println!("Not ELF");
        return Err(Status::LOAD_ERROR);
    }
    let mut min_vaddr = u64::MAX;
    let mut max_vaddr = 0u64;

    for i in 0..ehdr.e_phnum {
        let phdr = unsafe {
            &*(buffer_ptr.add(
                ehdr.e_phoff as usize
                    + i as usize * ehdr.e_phentsize as usize,
            ) as *const Elf64Phdr)
        };

        if phdr.p_type != PT_LOAD {
            continue;
        }

        min_vaddr = min_vaddr.min(phdr.p_vaddr);
        max_vaddr = max_vaddr.max(phdr.p_vaddr + phdr.p_memsz);
    }

    let start_addr = (min_vaddr & !0xFFF) as usize;
    let end_addr = ((max_vaddr + 0xFFF) & !0xFFF) as usize;

    let image_size = end_addr - start_addr;
    let pages = image_size / 0x1000;

    let address = uefi::boot::allocate_pages(
        uefi::boot::AllocateType::AnyPages,
        MemoryType::LOADER_DATA,
        pages,
    )
    .expect("Failed to reserve memory");

    let load_address = address.as_ptr() as usize;

    let load_bias = load_address as isize - start_addr as isize;


    for i in 0..ehdr.e_phnum {
        let phdr = unsafe {
            &*(buffer_ptr.add(
                ehdr.e_phoff as usize
                    + i as usize * ehdr.e_phentsize as usize,
            ) as *const Elf64Phdr)
        };

        if phdr.p_type != PT_LOAD {
            continue;
        }
        
        let dst = (phdr.p_vaddr as isize + load_bias) as *mut u8;

        let src = unsafe {
            buffer_ptr.add(phdr.p_offset as usize)
        };

        unsafe {
            core::ptr::copy_nonoverlapping(
                src,
                dst,
                phdr.p_filesz as usize,
            );

            if phdr.p_memsz > phdr.p_filesz {
                core::ptr::write_bytes(
                    dst.add(phdr.p_filesz as usize),
                    0,
                    (phdr.p_memsz - phdr.p_filesz) as usize,
                );
            }
        }
    }

    let base = load_address as u64;
    let entry = (ehdr.e_entry as isize + load_bias) as u64;

    Ok(RawModule {
        raw_ptr: buffer_ptr as u64,
        raw_len: size as u64,
        base: base,
        address: entry,
        len: image_size as u64,
        load_bias: load_bias as i64
    })
}