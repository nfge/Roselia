pub const ET_NONE: u16 = 0;
pub const ET_REL: u16  = 1;
pub const ET_EXEC: u16 = 2;
pub const ET_DYN: u16  = 3;
pub const ET_CORE: u16 = 4;

#[repr(C)]
#[derive(Debug,Clone, Copy)]
pub struct Elf64Ehdr {
    pub e_ident: [u8; 16],
    pub e_type: u16,
    pub e_machine: u16,
    pub e_version: u32,
    pub e_entry: u64,
    pub e_phoff: u64,
    pub e_shoff: u64,
    pub e_flags: u32,
    pub e_ehsize: u16,
    pub e_phentsize: u16,
    pub e_phnum: u16,
    pub e_shentsize: u16,
    pub e_shnum: u16,
    pub e_shstrndx: u16,
}
