use kernel_api::{
    elf::{elf64ehdr::Elf64Ehdr, elf64phdr::Elf64Phdr},
    module::RawModule,
};
use uefi::{
    CStr16, Status,
    boot::{self, MemoryType, ScopedProtocol, allocate_pages},
    cstr16, println,
    proto::media::{
        file::{self, Directory, File, FileAttribute, FileInfo, FileMode},
        fs::SimpleFileSystem,
    },
};

use crate::PT_LOAD;

pub mod init_gop;

pub fn get_kernel(
    filesys: &mut ScopedProtocol<SimpleFileSystem>,
) -> Result<(u64, usize, usize), uefi::Status> {
    let mut kernel_start_addr: usize;
    let mut kernel_pages: usize;
    let mut root = filesys.open_volume().expect("Failed to open volume");
    let kernel_name: &CStr16 = cstr16!("kernel.elf");
    let mut kernel = match root.open(&kernel_name, FileMode::Read, FileAttribute::empty()) {
        Ok(f) => match f.into_type() {
            Ok(file::FileType::Regular(file)) => file,
            _ => {
                println!("kernel.elf is not a regular file");
                return Err(Status::UNSUPPORTED);
            }
        },
        Err(_) => {
            println!("Failed to open kernel.elf");
            return Err(Status::NOT_FOUND);
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
        return Err(Status::LOAD_ERROR);
    }

    let mut last_allocated_start = 0usize;
    let mut last_allocated_end = 0usize;
    
    let mut kernel_min_start = usize::MAX;
    let mut kernel_max_end = 0usize;

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

        kernel_min_start = kernel_min_start.min(start_addr);
        kernel_max_end = kernel_max_end.max(end_addr);

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

    kernel_start_addr = kernel_min_start;
    kernel_pages = (kernel_max_end - kernel_min_start) / 0x1000;

    let entry = ehdr.e_entry;
    Ok((entry, kernel_start_addr, kernel_pages))
}

pub fn load_modules(
    file_system: &mut ScopedProtocol<SimpleFileSystem>,
) -> Result<(*mut RawModule, usize), uefi::Status> {
    let mut root = file_system.open_volume().unwrap();
    let mut modules_dir = root
        .open(
            cstr16!("modules"),
            uefi::proto::media::file::FileMode::Read,
            FileAttribute::empty(),
        )
        .expect("Failed to open directory")
        .into_directory()
        .expect("Not a directory");

    let mut buf = [0u8; 512];

    let mut module_count = 0;

    loop {
        match modules_dir.read_entry(&mut buf) {
            Ok(Some(entry)) => {
                if entry.attribute().contains(FileAttribute::DIRECTORY) {
                    continue;
                }
                if !has_extension(entry.file_name(), b".elf") && !has_extension(entry.file_name(), b".kmod") {
                    continue;
                }
                module_count += 1;
            }

            Ok(None) => {
                break;
            }

            Err(e) => {
                println!("Failed to read directory: {:?}", e);
                return Err(Status::LOAD_ERROR);
            }
        }
    }

    let ptr = if module_count == 0 {
        core::ptr::null_mut()
    } else {
        let size = module_count * core::mem::size_of::<RawModule>();

        let pages = (size + 0xFFF) / 0x1000;
        allocate_pages(
            uefi::boot::AllocateType::AnyPages,
            MemoryType::LOADER_DATA,
            pages,
        )
        .expect("Failed to allocate modules")
        .as_ptr() as *mut RawModule
    };

    let mut modules_dir = root
        .open(
            cstr16!("modules"),
            uefi::proto::media::file::FileMode::Read,
            FileAttribute::empty(),
        )
        .expect("Failed to open directory")
        .into_directory()
        .expect("Not a directory");

    let mut module_index = 0;

    loop {
        match modules_dir.read_entry(&mut buf) {
            Ok(Some(entry)) => {
                if entry.attribute().contains(FileAttribute::DIRECTORY) {
                    continue;
                }

                if module_index >= module_count {
                    break;
                }

                let name = entry.file_name();

                if !has_extension(name, b".elf") && !has_extension(name, b".kmod") {
                    continue;
                }

                let module = elf_loader::load_elf(name, file_system)?;

                unsafe {
                    ptr.add(module_index).write(module);
                }

                module_index += 1;
            }

            Ok(None) => {
                break;
            }

            Err(e) => {
                println!("Failed to read directory: {:?}", e);
                return Err(Status::LOAD_ERROR);
            }
        }
    }

    if module_index != module_count {
        println!(
            "Warning: expected {}, loaded {} modules",
            module_count, module_index
        );
    }
    Ok((ptr, module_count))
}

fn has_extension(name: &CStr16, extension: &[u8]) -> bool {
    let units = name.to_u16_slice();
    if units.len() < extension.len() {
        return false;
    }
    units[units.len() - extension.len()..].iter().zip(extension).all(|(&cu,&b)| cu <= 0x7F && (cu as u8).eq_ignore_ascii_case(&b))
}