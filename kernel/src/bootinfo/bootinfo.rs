use super::gop_table::gop_table;
use super::reset::ResetFn;
use super::time::GetTimeFn;
use super::variable::{GetVar,SetVar};

#[repr(C)]
pub struct BootInfo {
    pub framebuffer: gop_table,
    pub time: GetTimeFn,
    pub reset: ResetFn,
    pub set_var: SetVar,
    pub get_var: GetVar
}