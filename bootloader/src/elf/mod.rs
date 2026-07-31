// use elf_headers::elf64ehdr::Elf64Ehdr;
// use uefi::{CStr16, Status, boot::{self, MemoryType, allocate_pages}, cstr16, println, proto::media::file::{self, Directory, File, FileAttribute, FileInfo, FileMode}};

// pub fn get_ehdr<'a>(root: &mut Directory, file_name: &CStr16) -> Result<&'a Elf64Ehdr, uefi::Status> {
//     let kernel_name: &CStr16 = file_name;
//     let mut kernel = match root.open(&kernel_name, FileMode::Read, FileAttribute::empty()) {
//         Ok(f) => match f.into_type() {
//             Ok(file::FileType::Regular(file)) => file,
//             _ => {
//                 println!("{} is not a regular file", file_name);
//                 return Err(Status::UNSUPPORTED);
//             }
//         },
//         Err(_) => {
//             println!("Failed to open kernel.elf");
//             return Err(Status::NOT_FOUND);
//         }
//     };
//     let mut buf = [0u8; 512];
//     let info = kernel
//         .get_info::<FileInfo>(&mut buf)
//         .expect("Failed to get file info");

//     let size = info.file_size() as usize;

//     let pages = (size + 0xFFF) / 0x1000;

//     let buffer_ptr = allocate_pages(boot::AllocateType::AnyPages, MemoryType::LOADER_DATA, pages)
//         .expect("Failed to allocate pages")
//         .as_ptr();
//     let mut offset = 0;
//     let chunk = 64 * 1024;
//     while offset < size {
//         let end = (offset + chunk).min(size);
//         let slice =
//             unsafe { core::slice::from_raw_parts_mut(buffer_ptr.add(offset), end - offset) };
//         let read = kernel.read(slice).expect("Failed to read file");
//         if read == 0 {
//             break;
//         }
//         offset += read;
//     }
//     let ehdr = unsafe { &*(buffer_ptr as *const Elf64Ehdr) };
//     if &ehdr.e_ident[0..4] != b"\x7FELF" {
//         println!("Not ELF");
//         return Err(Status::LOAD_ERROR);
//     }
//     Ok(ehdr)
// }