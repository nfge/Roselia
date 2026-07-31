#[repr(C)]
#[derive(Clone, Copy)]
pub struct Elf64Dyn {
    pub d_tag: i64,
    pub d_un: crate::elf64dynval::Elf64DynVal,
}