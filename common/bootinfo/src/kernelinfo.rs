use core::ptr::NonNull;

#[repr(C)]
pub struct KernelInfo {
    pub start_address: usize,
    pub pages: usize,
    pub stack_info: StackInfo
}

#[repr(C)]
pub struct StackInfo {
    pub stack_ptr: NonNull<u8>,
    pub stack_pages: usize
}