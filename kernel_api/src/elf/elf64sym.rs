use core::slice;

pub const SHN_UNDEF: u16 = 0;
pub const SHN_ABS:u16 = 0xFFF1;

pub const DT_SYMTAB: i64 = 6;
pub const DT_STRTAB: i64 = 5;
pub const DT_STRSZ: i64 = 10;

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

pub unsafe fn sym_name(strtab: *const u8, st_name: u32) -> &'static str {
    let start = unsafe {strtab.add(st_name as usize)};
    let mut len = 0;
    while unsafe {*start.add(len) != 0} { len += 1; }
    unsafe {core::str::from_utf8_unchecked(slice::from_raw_parts(start, len))}
}