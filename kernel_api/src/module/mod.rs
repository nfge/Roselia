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
pub struct RawModule {
    pub raw_ptr: u64,
    pub raw_len: u64,
    pub base: u64,
    pub address: u64,
    pub len: u64,
    pub load_bias: i64
}

#[repr(C)]
pub struct RawModules {
    pub ptr: *mut RawModule,
    pub count: usize
}

#[repr(C)]
pub struct Buffer {
    pub ptr: *mut u8,
    pub len: usize
}