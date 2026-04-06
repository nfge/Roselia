use uefi::runtime::reset;

pub fn reset_fn(reset_type:uefi::runtime::ResetType, status: uefi::Status, data: Option<&[u8]>) -> ! {
    reset(reset_type, status, data);
}


pub type ResetFn = fn(uefi::runtime::ResetType,uefi::Status,Option<&[u8]>) -> !;