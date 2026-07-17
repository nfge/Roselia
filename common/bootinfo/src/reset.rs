use uefi::runtime::reset;

#[allow(improper_ctypes_definitions)]
pub extern "win64" fn reset_fn(reset_type:uefi::runtime::ResetType, status: uefi::Status, data: Option<&[u8]>) -> ! {
    x86_64::instructions::interrupts::without_interrupts(|| {
        reset(reset_type, status, data);
    })
}

#[allow(improper_ctypes_definitions)]
pub type ResetFn = extern "win64" fn(uefi::runtime::ResetType,uefi::Status,Option<&[u8]>) -> !;