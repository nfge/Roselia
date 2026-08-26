
#[repr(transparent)]
#[derive(Clone, Copy)]
pub struct SymAddr(pub *const ());

unsafe impl Sync for SymAddr {}

#[repr(C)]
pub struct KernelSymbol {
    pub name: &'static str,
    pub addr: SymAddr
}