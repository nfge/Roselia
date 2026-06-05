use super::variable::{SetVar,GetVar};

use super::gop_table::gop_table;
use super::reset::ResetFn;
use super::time::GetTimeFn;

#[repr(C)]
pub struct BootInfo {
    pub gop: gop_table,
    pub time: GetTimeFn,
    pub reset: ResetFn,
    pub set_var: SetVar,
    pub get_var: GetVar
}