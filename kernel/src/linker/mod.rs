use alloc::slice;
use kernel_api::{
    elf::{
        elf64dyn::{DT_NULL, DT_RELA, DT_RELAENT, DT_RELASZ, Elf64Dyn},
        elf64ehdr::Elf64Ehdr,
        elf64phdr::{Elf64Phdr, PF_W, PF_X, PT_DYNAMIC, PT_LOAD},
        elf64rela::{Elf64Rela, R_X86_64_64, R_X86_64_GLOB_DAT, R_X86_64_JUMP_SLOT, R_X86_64_RELATIVE}, elf64sym::{Elf64Sym, SHN_UNDEF, sym_name},
    },
    module::RawModule,
    symbol::{KernelSymbol, SymAddr},
};

use x86_64::{
    structures::paging::{Mapper, Page, PageTableFlags, Size4KiB},
    VirtAddr,
};

use crate::{log_err, log_fail, terminal::kprint};

pub static KERNEL_EXPORTS: &[KernelSymbol] = &[KernelSymbol {
    name: "kprint",
    addr: SymAddr(kprint as *const ()),
}];


pub struct Linker;

impl Linker {
    pub unsafe fn relocate_module(module: &RawModule, symtab: *const Elf64Sym, strtab: *const u8) {
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
            return;
        };
        let dyn_entries = unsafe {
            slice::from_raw_parts(
                file.as_ptr().add(dyn_phdr.p_offset as usize) as *const Elf64Dyn,
                dyn_phdr.p_filesz as usize / size_of::<Elf64Dyn>(),
            )
        };
        let (mut rela_vaddr, mut rela_size, mut rela_ent) = (None, 0usize, size_of::<Elf64Rela>());
        for d in dyn_entries {
            match d.d_tag {
                DT_RELA => rela_vaddr = Some(d.d_un.d_val),
                DT_RELASZ => rela_size = d.d_un.d_val as usize,
                DT_RELAENT => rela_ent = d.d_un.d_val as usize,
                DT_NULL => break,
                _ => {}
            }
        }
        let Some(rela_vaddr) = rela_vaddr else { return };
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
                    unsafe {target.write_unaligned(value)};
                }

                R_X86_64_64 | R_X86_64_GLOB_DAT | R_X86_64_JUMP_SLOT => {
                    let sym = unsafe {&*symtab.add(r.sym() as usize)};

                    let s = if sym.st_shndx != SHN_UNDEF {
                        (module.load_bias + sym.st_value as i64) as u64
                    } else {
                        let name = unsafe {sym_name(strtab, sym.st_name)};
                        resolve_kernel_symbol(name)
                            .unwrap_or_else(|| {
                                panic!("Symbol not found: {name}")
                            })
                    };

                    let value = match r.reloc_type() {
                        R_X86_64_JUMP_SLOT => s,                
                        _ => s.wrapping_add(r.r_addend as u64), 
                    };
                    unsafe {target.write_unaligned(value)};
                }

                other => log_fail!("Not supported relocation type: {other}")
            }
        }
    }
    pub unsafe fn protect_module(module: &RawModule, mapper: &mut impl Mapper<Size4KiB>) {
        let file = unsafe {
            slice::from_raw_parts(module.raw_ptr as *const u8, module.raw_len as usize)
        };
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
                    mapper
                        .update_flags(page, flags)
                        .expect("Fail")
                        .flush();
                }
            }
        }
    }
    pub unsafe fn parse_symtab_strtab(module: &RawModule) -> Option<(*const Elf64Sym, *const u8)> {
        let file = unsafe {
            slice::from_raw_parts(module.raw_ptr as *const u8, module.raw_len as usize)
        };
        let ehdr = unsafe { &*(file.as_ptr() as *const Elf64Ehdr) };
        let phdrs = unsafe {
            slice::from_raw_parts(
                file.as_ptr().add(ehdr.e_phoff as usize) as *const Elf64Phdr,
                ehdr.e_phnum as usize,
            )
        };
        let dyn_phdr = phdrs.iter().find(|p| p.p_type == PT_DYNAMIC)?;
        let dyn_entries = unsafe {
            slice::from_raw_parts(
                file.as_ptr().add(dyn_phdr.p_offset as usize) as *const Elf64Dyn,
                dyn_phdr.p_filesz as usize / size_of::<Elf64Dyn>(),
            )
        };

        let (mut symtab_vaddr, mut strtab_vaddr) = (None, None);
        for d in dyn_entries {
            match d.d_tag {
                DT_SYMTAB => symtab_vaddr = Some(d.d_un.d_val),
                DT_STRTAB => strtab_vaddr = Some(d.d_un.d_val),
                DT_NULL => break,
                _ => {}
            }
        }

        let symtab = (module.load_bias + symtab_vaddr? as i64) as *const Elf64Sym;
        let strtab = (module.load_bias + strtab_vaddr? as i64) as *const u8;
        Some((symtab, strtab))
    }
}

fn resolve_kernel_symbol(name: &str) -> Option<u64> {
    KERNEL_EXPORTS.iter().find(|s| s.name == name).map(|s| s.addr.0 as u64)
}