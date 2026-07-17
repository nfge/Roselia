#[allow(improper_ctypes_definitions)]
pub extern "win64" fn set_variable(name:&uefi::CStr16,vendor:&uefi::runtime::VariableVendor,attributes:uefi::runtime::VariableAttributes, data: &[u8]) -> Result<(), uefi::Error> {
    return uefi::runtime::set_variable(name, vendor, attributes, data);
}

#[allow(improper_ctypes_definitions)]
pub extern "win64" fn get_variable<'buf>(name: &uefi::CStr16, vendor: &uefi::runtime::VariableVendor, buf: &'buf mut [u8]) -> Result<(&'buf [u8], uefi::runtime::VariableAttributes), uefi::Error<Option<usize>>> {
    return uefi::runtime::get_variable(name, vendor, buf).map(|(data_mut, attrs)| (&*data_mut, attrs));
}
#[allow(improper_ctypes_definitions)]
pub type SetVar = extern "win64" fn(name:&uefi::CStr16,vendor:&uefi::runtime::VariableVendor,attributes:uefi::runtime::VariableAttributes, data: &[u8]) -> Result<(), uefi::Error>;

#[allow(improper_ctypes_definitions)]
pub type GetVar = for<'buf> extern "win64" fn(name: &uefi::CStr16, vendor: &uefi::runtime::VariableVendor, buf: &'buf mut [u8]) -> Result<(&'buf [u8], uefi::runtime::VariableAttributes), uefi::Error<Option<usize>>>;