#![no_std]
#![no_main]

pub mod gop_table;
pub mod reset;
pub mod time;
pub mod variable;


#[repr(C)]
pub struct BootInfo {
    pub gop: gop_table::gop_table,
    pub time: time::GetTimeFn,
    pub reset: reset::ResetFn,
    pub set_var: variable::SetVar,
    pub get_var: variable::GetVar,
    pub memory_map: uefi::mem::memory_map::MemoryMapOwned,
}