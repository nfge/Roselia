pub const R_X86_64_RELATIVE: u32 = 8; 
pub const R_X86_64_64: u32       = 1; 
pub const R_X86_64_GLOB_DAT: u32 = 6;
pub const R_X86_64_JUMP_SLOT: u32 = 7;

#[repr(C)]
#[derive(Debug,Clone, Copy)]
pub struct Elf64Rela {
    pub r_offset: u64,
    pub r_info: u64,
    pub r_addend: i64
}

impl Elf64Rela {
    pub const fn reloc_type(&self) -> u32 { (self.r_info & 0xffff_ffff) as u32 }
    pub const fn sym(&self) -> u32 { (self.r_info >> 32) as u32 }
}