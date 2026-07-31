#[repr(C)]
#[derive(Debug,Clone, Copy)]
pub struct Elf64Sym {
    pub st_name: u32,
    pub st_info: u8,
    pub st_other: u8,
    pub st_shndx: u16,
    pub st_value: u64,
    pub st_size: u64
}

impl Elf64Sym {
    pub const fn bind(&self) -> u8 { self.st_info >> 4 }
    pub const fn sym_type(&self) -> u8 { self.st_info & 0xf }
}
