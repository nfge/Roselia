use alloc::slice;
use kernel_api::{
    elf::{
        elf64dyn::{DT_JMPREL, DT_NULL, DT_PLTRELSZ, DT_RELA, DT_RELAENT, DT_RELASZ, Elf64Dyn},
        elf64ehdr::Elf64Ehdr,
        elf64phdr::{Elf64Phdr, PF_W, PF_X, PT_DYNAMIC, PT_LOAD},
        elf64rela::{
            Elf64Rela, R_X86_64_64, R_X86_64_GLOB_DAT, R_X86_64_JUMP_SLOT, R_X86_64_RELATIVE,
        },
        elf64shdr::Elf64Shdr,
        elf64sym::{Elf64Sym, SHN_ABS, SHN_UNDEF, sym_name},
    },
    module::RawModule,
    symbol::{KernelSymbol, SymAddr},
};

use utils::serial_println;
use x86_64::{
    VirtAddr,
    structures::paging::{Mapper, Page, PageTableFlags, Size4KiB},
};

use crate::{linker::error::RelocateError, log_debug, log_fail, log_info, module::KERNEL_EXPORTS};

pub mod error;

pub struct Linker;

impl Linker {
    pub unsafe fn relocate_module(
        module: &RawModule,
        symtab: *const Elf64Sym,
        strtab: *const u8,
    ) -> Result<(), RelocateError> {
        let file =
            unsafe { slice::from_raw_parts(module.raw_ptr as *const u8, module.raw_len as usize) };
        let ehdr = unsafe { &*(file.as_ptr() as *const Elf64Ehdr) };
        let phdrs = unsafe {
            slice::from_raw_parts(
                file.as_ptr().add(ehdr.e_phoff as usize) as *const Elf64Phdr,
                ehdr.e_phnum as usize,
            )
        };
        let Some(dyn_phdr) = phdrs.iter().find(|p| p.p_type == PT_DYNAMIC) else {
            log_fail!("in module {}, no PT_DYNAMIC", module.address);
            serial_println!("in module {}, no PT_DYNAMIC", module.address);
            return Err(RelocateError::NoPTDYNAMIC);
        };
        let dyn_entries = unsafe {
            slice::from_raw_parts(
                file.as_ptr().add(dyn_phdr.p_offset as usize) as *const Elf64Dyn,
                dyn_phdr.p_filesz as usize / size_of::<Elf64Dyn>(),
            )
        };
        let (mut rela_vaddr, mut rela_size, mut rela_ent) = (None, 0usize, size_of::<Elf64Rela>());
        let (mut jmprel_vaddr, mut jmprel_size) = (None, 0usize);
        for d in dyn_entries {
            match d.d_tag {
                DT_RELA => rela_vaddr = Some(d.d_un.d_val),
                DT_RELASZ => rela_size = d.d_un.d_val as usize,
                DT_RELAENT => rela_ent = d.d_un.d_val as usize,
                DT_JMPREL => jmprel_vaddr = Some(d.d_un.d_val),
                DT_PLTRELSZ => jmprel_size = d.d_un.d_val as usize,
                DT_NULL => break,
                _ => {}
            }
        }
        if rela_vaddr.is_none() {
            rela_vaddr = jmprel_vaddr;
            rela_size = jmprel_size;
        }

        let Some(rela_vaddr) = rela_vaddr else {
            log_fail!("in module {}, no relocation table", module.address);
            serial_println!("in module {}, no relocation table", module.address);
            return Err(RelocateError::NORELDATA);
        };
        let count = rela_size / rela_ent;

        let rela_table = unsafe {
            slice::from_raw_parts(
                (module.load_bias + rela_vaddr as i64) as *const Elf64Rela,
                count,
            )
        };
        for r in rela_table {
            let target = (module.load_bias + r.r_offset as i64) as *mut u64;

            match r.reloc_type() {
                R_X86_64_RELATIVE => {
                    let value = (module.load_bias + r.r_addend) as u64;
                    unsafe { target.write_unaligned(value) };
                }

                R_X86_64_64 | R_X86_64_GLOB_DAT | R_X86_64_JUMP_SLOT => {
                    let sym = unsafe { &*symtab.add(r.sym() as usize) };

                    let name = unsafe { sym_name(strtab, sym.st_name) };

                    let s = match sym.st_shndx {
                        SHN_UNDEF => {
                            if cfg!(debug_assertions) {
                                serial_println!("Resolving external: {}\n", name);
                            }
                            log_info!("Resolving external: {}\n", name);

                            match resolve_kernel_symbol(name) {
                                Some(s) => s,
                                None => {
                                    serial_println!("Symbol not found: {}", name);
                                    log_fail!("Symbol not found: {}", name);
                                    return Err(RelocateError::SymbolNotFound);
                                }
                            }
                        }

                        SHN_ABS => sym.st_value,

                        _ => module.load_bias as u64 + sym.st_value,
                    };

                    let value = match r.reloc_type() {
                        R_X86_64_JUMP_SLOT => s,
                        _ => s.wrapping_add(r.r_addend as u64),
                    };
                    unsafe { target.write_unaligned(value) };
                }

                other => log_fail!("Not supported relocation type: {other}\n"),
            }
        }
        Ok(())
    }
    pub unsafe fn protect_module(module: &RawModule, mapper: &mut impl Mapper<Size4KiB>) {
        let file =
            unsafe { slice::from_raw_parts(module.raw_ptr as *const u8, module.raw_len as usize) };
        let ehdr = unsafe { &*(file.as_ptr() as *const Elf64Ehdr) };
        let phdrs = unsafe {
            slice::from_raw_parts(
                file.as_ptr().add(ehdr.e_phoff as usize) as *const Elf64Phdr,
                ehdr.e_phnum as usize,
            )
        };

        for ph in phdrs.iter().filter(|p| p.p_type == PT_LOAD) {
            let start = (module.load_bias + ph.p_vaddr as i64) as u64;
            let end = start + ph.p_memsz;

            let mut flags = PageTableFlags::PRESENT;
            if ph.p_flags & PF_W != 0 {
                flags |= PageTableFlags::WRITABLE;
            }
            if ph.p_flags & PF_X == 0 {
                flags |= PageTableFlags::NO_EXECUTE;
            }

            let start_page = Page::<Size4KiB>::containing_address(VirtAddr::new(start));
            let end_page = Page::<Size4KiB>::containing_address(VirtAddr::new(end - 1));

            for page in Page::range_inclusive(start_page, end_page) {
                unsafe {
                    mapper.update_flags(page, flags).expect("Fail").flush();
                }
            }
        }
    }
    pub unsafe fn parse_dyntab_dyntab(module: &RawModule) -> Option<(*const Elf64Sym, *const u8)> {
        let file =
            unsafe { slice::from_raw_parts(module.raw_ptr as *const u8, module.raw_len as usize) };

        let ehdr = unsafe { &*(file.as_ptr() as *const Elf64Ehdr) };

        let shdrs = unsafe {
            slice::from_raw_parts(
                file.as_ptr().add(ehdr.e_shoff as usize) as *const Elf64Shdr,
                ehdr.e_shnum as usize,
            )
        };

        let shstrtab = shdrs.get(ehdr.e_shstrndx as usize)?;

        let shstrtab_data = unsafe {
            slice::from_raw_parts(
                file.as_ptr().add(shstrtab.sh_offset as usize),
                shstrtab.sh_size as usize,
            )
        };

        let mut symtab = None;
        let mut strtab = None;

        for shdr in shdrs {
            let start = shdr.sh_name as usize;

            if start >= shstrtab_data.len() {
                continue;
            }

            let end = shstrtab_data[start..]
                .iter()
                .position(|&b| b == 0)
                .map(|x| start + x)
                .unwrap_or(shstrtab_data.len());

            let name = &shstrtab_data[start..end];

            match name {
                b".dynsym" => symtab = Some(shdr),
                b".dynstr" => strtab = Some(shdr),
                _ => {}
            }
        }

        let symtab = symtab?;
        let strtab = strtab?;

        if symtab.sh_entsize != size_of::<Elf64Sym>() as u64 {
            return None;
        }

        let symtab_ptr = unsafe { file.as_ptr().add(symtab.sh_offset as usize) as *const Elf64Sym };

        let strtab_ptr = unsafe { file.as_ptr().add(strtab.sh_offset as usize) };

        Some((symtab_ptr, strtab_ptr))
    }
}

fn resolve_kernel_symbol(name: &str) -> Option<u64> {
    KERNEL_EXPORTS
        .lock()
        .iter()
        .find(|s| s.name == name)
        .map(|s| s.addr.0 as u64)
}
