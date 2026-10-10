use kernel_api::module::raw::RawModule;
use uefi::{CStr16, Status, boot::{MemoryType, ScopedProtocol, allocate_pages}, cstr16, println, proto::media::{file::{File, FileAttribute}, fs::SimpleFileSystem}};

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

                let module = elf_loader::load_elf(&mut modules_dir, name)?;

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