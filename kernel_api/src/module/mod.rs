pub mod raw;

pub const ACCEPT_ARGS: u32 = 1 << 0;

#[repr(C)]
pub struct Module {
    pub entry_fn: *const (),
    pub address: u64,
    pub info: ModuleInfo
}

unsafe impl Send for Module {}
unsafe impl Sync for Module {}



#[repr(C)]
#[derive(Clone, Copy)]
pub struct ModuleInfo {
    pub name: [u8; 32],

    pub module_version: u32,
    pub abi_version: u32,

    pub magic: u64,
    pub flags: u32
}


#[repr(C)]
#[derive(Copy, Clone)]
pub struct ModuleArgs {
    pub argc: u64,
    pub argv: *const *const u8
}





#[repr(C)]
pub struct Buffer {
    pub ptr: *mut u8,
    pub len: usize
}