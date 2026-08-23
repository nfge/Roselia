pub const PT_NULL: u32    = 0;
pub const PT_LOAD: u32    = 1;
pub const PT_DYNAMIC: u32 = 2;
pub const PT_INTERP: u32  = 3;
pub const PT_NOTE: u32    = 4;
pub const PT_PHDR: u32    = 6;
pub const PT_TLS: u32     = 7;

pub const PF_X: u32 = 1;
pub const PF_W: u32 = 2; 
pub const PF_R: u32 = 4; 

#[repr(C)]
#[derive(Debug,Clone, Copy)]
pub struct Elf64Phdr {
    pub p_type: u32,
    pub p_flags: u32,
    pub p_offset: u64,
    pub p_vaddr: u64,
    pub p_paddr: u64,
    pub p_filesz: u64,
    pub p_memsz: u64,
    pub p_align: u64,
}