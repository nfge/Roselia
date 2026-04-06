use super::gop_table::gop_table;
use super::reset::ResetFn;
use super::time::GetTimeFn;
use super::fat32_table::FAT32;

#[repr(C)]
pub struct BootInfo {
    pub framebuffer: gop_table,
    pub time: GetTimeFn,
    pub reset: ResetFn,
    // pub fs: FAT32
}