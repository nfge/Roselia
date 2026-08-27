use core::{
    ops::Add,
    panic::{self, PanicInfo},
    slice,
};

use alloc::vec::Vec;
use kernel_api::{
    elf::{
        elf64ehdr::Elf64Ehdr,
        elf64phdr::{Elf64Phdr, PF_R, PF_X, PT_LOAD},
        elf64shdr::Elf64Shdr,
    },
    module::{Module, ModuleInfo, RawModule},
    symbol::{KernelSymbol, SymAddr},
};
use pci::{find_by_class, find_by_id};

use crate::{
    cpu::cpuinfo::get_cpu, kprintln, linker::Linker, log_info, memory::{kalloc, kfree}, module::error::LoadError, ramfs::read_file, terminal::{kprint, kprintln}
};

mod error;

pub static KERNEL_EXPORTS: &[KernelSymbol] = &[
    KernelSymbol {
        name: "kprint",
        addr: SymAddr(kprint as *const ()),
    },
    KernelSymbol {
        name: "kprintln",
        addr: SymAddr(kprintln as *const ()),
    },
    KernelSymbol {
        name: "module_panic",
        addr: SymAddr(module_panic as *const ()),
    },
    KernelSymbol {
        name: "kalloc",
        addr: SymAddr(kalloc as *const ()),
    },
    KernelSymbol {
        name: "kfree",
        addr: SymAddr(kfree as *const ()),
    },
    KernelSymbol {
        name: "read",
        addr: SymAddr(read_file as *const ())
    },
    KernelSymbol {
        name: "pci_find_by_id",
        addr: SymAddr(find_by_id as *const ())
    },
    KernelSymbol {
        name: "pci_find_by_class",
        addr: SymAddr(find_by_class as *const ())
    },
    KernelSymbol {
        name: "get_cpu",
        addr: SymAddr(get_cpu as *const ())
    }
];

pub unsafe fn load_module(module: &RawModule) -> Result<Module, LoadError> {
    let file =
        unsafe { slice::from_raw_parts(module.raw_ptr as *const u8, module.raw_len as usize) };
    let ehdr = unsafe { &*(file.as_ptr() as *const Elf64Ehdr) };

    let info = unsafe { read_module_info(module).unwrap() };

    if info.magic != 0x524F_5345_4C49_4100 {
        return Err(LoadError::InvalidMagic);
    }
    if info.abi_version != 1 {
        return Err(LoadError::InvalidAbiVersion);
    }

    let (symtab, strtab) = unsafe { Linker::parse_dyntab_dyntab(module).unwrap() };
    unsafe { 
        let Ok(_) = Linker::relocate_module(module, symtab, strtab) else {
            return Err(LoadError::RelocateError)
        };
    };

    // unsafe { Linker::protect_module(module, mapper) };

    let entry_addr = (module.load_bias + ehdr.e_entry as i64) as u64;

    if !is_in_executable_segment(module, entry_addr) {
        return Err(LoadError::EntryNotExecutable);
    }
    log_info!(
        "Successful loaded module {} {}\n",
        core::str::from_utf8(
            &info.name[..info
                .name
                .iter()
                .position(|&c| c == 0)
                .unwrap_or(info.name.len())]
        )
        .unwrap(),
        module.address
    );
    Ok(Module {
        entry_fn: entry_addr as *const (),
        address: module.address,
        info,
    })
}

fn is_in_executable_segment(module: &RawModule, addr: u64) -> bool {
    let file =
        unsafe { slice::from_raw_parts(module.raw_ptr as *const u8, module.raw_len as usize) };
    let ehdr = unsafe { &*(file.as_ptr() as *const Elf64Ehdr) };
    let phdrs = unsafe {
        slice::from_raw_parts(
            file.as_ptr().add(ehdr.e_phoff as usize) as *const Elf64Phdr,
            ehdr.e_phnum as usize,
        )
    };
    phdrs.iter().any(|ph| {
        ph.p_type == PT_LOAD && ph.p_flags & PF_X != 0 && {
            let start = (module.load_bias + ph.p_vaddr as i64) as u64;
            (start..start + ph.p_memsz).contains(&addr)
        }
    })
}

pub unsafe fn read_module_info(module: &RawModule) -> Option<ModuleInfo> {
    let file =
        unsafe { slice::from_raw_parts(module.raw_ptr as *const u8, module.raw_len as usize) };
    let ehdr = unsafe { &*(file.as_ptr() as *const Elf64Ehdr) };

    let shdrs = unsafe {
        core::slice::from_raw_parts(
            module.raw_ptr.add(ehdr.e_shoff) as *const Elf64Shdr,
            ehdr.e_shnum as usize,
        )
    };

    let shstrtab = &shdrs[ehdr.e_shstrndx as usize];
    let strtab: &[u8] = unsafe {
        core::slice::from_raw_parts(
            (module.raw_ptr as *const u8).add(shstrtab.sh_offset as usize),
            shstrtab.sh_size as usize,
        )
    };

    for shdr in shdrs {
        let start_name = shdr.sh_name as usize;
        let end_name = strtab[start_name..]
            .iter()
            .position(|&b| b == 0)
            .map(|x| start_name + x)
            .unwrap_or(strtab.len());
        let name = &strtab[start_name..end_name];
        if name == b".module_info" {
            let module_info_ptr =
                unsafe { (module.raw_ptr as *const u8).add(shdr.sh_offset as usize) };
            let module_info = unsafe { *(module_info_ptr as *const ModuleInfo) };
            return Some(module_info);
        }
    }
    None
}

fn module_panic(info: &PanicInfo) {
    kprintln!("{}", info);
}
