#[repr(C)]
#[derive(Clone, Copy)]
pub union Elf64DynVal {
    pub d_val: u64,
    pub d_ptr: u64,
}