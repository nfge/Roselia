#[repr(C)]
pub struct KernelInfo {
    pub start_address: usize,
    pub pages: usize
}