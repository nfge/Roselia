use uefi::runtime::{Time, TimeCapabilities};

use crate::{GET_VAR_FN, RESET_FN, SET_VAR_FN, TIME_FN};

pub fn reset(reset_type: uefi::runtime::ResetType, status: uefi::Status, data: Option<&[u8]>) -> ! {
    unsafe { RESET_FN.unwrap()(reset_type, status, data) };
}

pub fn get_time() -> Result<(Time, TimeCapabilities), uefi::Error<()>> {
    unsafe { TIME_FN.unwrap()() }
}
pub fn set_uefi_var(
    name: &uefi::CStr16,
    vendor: &uefi::runtime::VariableVendor,
    attributes: uefi::runtime::VariableAttributes,
    data: &[u8],
) -> Result<(), uefi::Error> {
    unsafe { SET_VAR_FN.unwrap()(name, vendor, attributes, data) }
}
pub fn get_uefi_var<'buf>(
    name: &uefi::CStr16,
    vendor: &uefi::runtime::VariableVendor,
    buf: &'buf mut [u8],
) -> Result<(&'buf [u8], uefi::runtime::VariableAttributes), uefi::Error<Option<usize>>> {
    return unsafe {GET_VAR_FN.unwrap()(name,vendor,buf)};
}

