#[repr(C)]
pub struct ModuleInfo {
    pub name: [u8; 32],

    pub module_version: u32,
    pub abi_version: u32,

    pub magic: u64,
    pub version: u32,
    pub flags: u32
}


#[repr(C)]
pub struct Module {
    pub address: u64,
    pub len: u64
}

#[repr(C)]
pub struct Modules {
    pub ptr: *mut Module,
    pub count: usize
}