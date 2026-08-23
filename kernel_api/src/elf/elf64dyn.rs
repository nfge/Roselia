pub const DT_NULL: i64  = 0; 
pub const DT_RELA: i64  = 7; 
pub const DT_RELASZ: i64 = 8; 
pub const DT_RELAENT: i64 = 9;
pub const DT_SYMTAB: i64 = 6;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct Elf64Dyn {
    pub d_tag: i64,
    pub d_un: crate::elf::elf64dynval::Elf64DynVal,
}