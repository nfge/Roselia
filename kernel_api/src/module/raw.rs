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