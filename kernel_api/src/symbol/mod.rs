
#[repr(transparent)]
#[derive(Clone, Copy)]
pub struct SymAddr(pub usize);

#[repr(C)]
pub struct KernelSymbol {
    pub name: &'static str,
    pub addr: SymAddr
}