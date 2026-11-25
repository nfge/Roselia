use core::ffi::CStr;

#[cfg(feature = "alloc")]
use alloc::vec::Vec;

use crate::symbol::KernelSymbol;

pub mod raw;


pub const ABI_V1: u32 = 1;
pub const MAGIC: u64 = 0x524F_5345_4C49_4100;

bitflags::bitflags! {
    #[derive(Debug)]
    pub struct ModuleFlags: u32 {
        const ACCEPT_ARGS = 1 << 0;
        const NO_ENTRY = 1 << 1;
    }
}

pub type ModuleId = usize;

#[repr(C)]
pub struct Module {
    pub id: ModuleId,
    pub entry_fn: Option<*const ()>,
    pub address: u64,
    pub info: ModuleInfo
}

unsafe impl Send for Module {}
unsafe impl Sync for Module {}



#[repr(C)]
#[derive(Clone, Copy)]
pub struct ModuleInfo {
    pub name: [u8; 32],

    pub mtype: ModuleType,

    pub module_version: f32,
    pub abi_version: u32,

    pub magic: u64,
    pub flags: u32
}

#[repr(C)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ModuleType {
    Executable,
    Driver
}


#[repr(C)]
#[derive(Copy, Clone)]
pub struct ModuleArgs {
    pub argc: u64,
    pub argv: *const *const u8
}

impl ModuleArgs {
    #[cfg(feature = "alloc")]
    pub fn to_vec(&self) -> Option<Vec<&str>> {
        let mut argsv: Vec<&str> = Vec::new();
        let args = unsafe {core::slice::from_raw_parts(self.argv,self.argc as usize)};
        for i in 0..self.argc {
            unsafe {
                match CStr::from_ptr(args[i as usize].cast()).to_str().map(|s| argsv.push(s)) {
                    Ok(_) => {},
                    Err(_) => {return None}
                }
            }
        }
        return Some(argsv)
    } 
}

#[repr(C)]
pub struct ModuleExport {
    pub name: *const u8,
    pub nlen: usize,
    pub address: *const ()
}

unsafe impl Sync for ModuleExport {}