#![no_std]
#![no_main]

pub mod gop_table;
pub mod reset;
pub mod time;
pub mod variable;


#[repr(C)]
pub struct BootInfo {
    pub gop: gop_table::gop_table,
    pub time: *const (),
    pub reset: *const (),
    pub set_var: *const (),
    pub get_var: *const (),
    pub memory_map: uefi::mem::memory_map::MemoryMapOwned,
}