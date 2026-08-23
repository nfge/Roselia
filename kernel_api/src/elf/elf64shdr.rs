pub const SHT_NULL: u32     = 0;
pub const SHT_PROGBITS: u32 = 1;
pub const SHT_SYMTAB: u32   = 2;
pub const SHT_STRTAB: u32   = 3;
pub const SHT_RELA: u32     = 4;
pub const SHT_DYNAMIC: u32  = 6;
pub const SHT_NOBITS: u32   = 8; 
pub const SHT_DYNSYM: u32   = 11;

pub const SHF_WRITE: u64     = 0x1;
pub const SHF_ALLOC: u64     = 0x2; 
pub const SHF_EXECINSTR: u64 = 0x4;


#[repr(C)]
#[derive(Debug,Clone, Copy)]
pub struct Elf64Shdr {
    pub sh_name: u32,
    pub sh_type: u32,
    pub sh_flags: u64,
    pub sh_addr: u64,
    pub sh_offset: u64,
    pub sh_size: u64,       
    pub sh_link: u32,       
    pub sh_info: u32,       
    pub sh_addralign: u64,  
    pub sh_entsize: u64, 
}