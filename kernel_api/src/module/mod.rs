#[repr(C)]
pub struct ModuleInfo {
    pub name: [u8; 32],

    pub module_version: u32,
    pub abi_version: u32,

    pub magic: u64,
    pub version: u32,
    pub flags: u32
}