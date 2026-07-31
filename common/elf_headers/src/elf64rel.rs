#[repr(C)]
#[derive(Debug,Clone, Copy)]
pub struct Elf64Rel {
    pub r_offset: u64,
    pub r_info: u64
}