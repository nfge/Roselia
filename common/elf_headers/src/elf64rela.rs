#[repr(C)]
#[derive(Debug,Clone, Copy)]
pub struct Elf64Rela {
    pub r_offset: u64,
    pub r_info: u64,
    pub r_addend: i64
}

impl Elf64Rela {
    pub const fn sym(&self) -> u32 { (self.r_info >> 32) as u32 }
    pub const fn reloc_type(&self) -> u32 { (self.r_info & 0xffff_ffff) as u32 }
}