use uefi::proto::console::gop::ModeInfo;

#[repr(C)]
pub struct gop_table {
    pub framebuffer_ptr: *mut u8,
    pub size: usize,
    pub width: usize,
    pub height: usize,
    pub stride: usize,
    pub mode_info: ModeInfo
}