use core::panic;

use acpi_tables::{
    dsdt::Dsdt,
    fadt::Fadt,
    func::{SLP_EN, SLP_TYP_SHIFT, find_s5_sleep_type},
    get_table,
    rsdp::Rsdp,
    sdtheader::SdtHeader,
    xsdt::Xsdt,
};
use alloc::format;
use bootinfo::time::KernelTime;
use utils::serial_println;
use x86::io::outb;
use x86_64::instructions::hlt;

use crate::{ACPI_TABLE, GET_VAR_FN, SET_VAR_FN, TIME_FN};

pub unsafe fn reset() -> ! {
    let fadt_ptr = unsafe { get_table::<Fadt>(ACPI_TABLE.unwrap(), b"FACP").unwrap() };
    let fadt = unsafe { &*fadt_ptr };
    match fadt.reset_reg.address_space {
        0 => unsafe {
            (fadt.reset_reg.address as *mut u8).write_volatile(fadt.reset_value);
        },
        1 => unsafe {
            outb(fadt.reset_reg.address as u16, fadt.reset_value);
        },
        _ => {
            panic!("Unsupported Address Space");
        }
    }
    loop {
        hlt();
    }
}
pub unsafe fn s5_soft_off() -> ! {
    use x86_64::instructions::port::Port;
    let fadt_ptr = unsafe { get_table::<Fadt>(ACPI_TABLE.unwrap(), b"FACP").unwrap() };
    let fadt = unsafe { &*fadt_ptr };
    let dsdt_addr = if fadt.x_dsdt != 0 {
        fadt.x_dsdt as usize
    } else {
        fadt.dsdt as usize
    };
    let dsdt = unsafe {&*(dsdt_addr as *const Dsdt)};

    let (slp_typa, slp_typb) = find_s5_sleep_type(unsafe {dsdt.aml_bytes()}).unwrap();
    let mut a: Port<u16> = Port::new(fadt.pm1a_control_block as u16);
    unsafe {a.write(((slp_typa as u16) << SLP_TYP_SHIFT) | SLP_EN)};
    if fadt.pm1b_control_block != 0 {
        let mut b: Port<u16> = Port::new(fadt.pm1b_control_block as u16);
        unsafe {b.write(((slp_typb as u16) << SLP_TYP_SHIFT) | SLP_EN)};
    }
    loop {
        hlt();
    }
}

pub fn get_time() -> Result<KernelTime, uefi::Error<()>> {
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
    return unsafe { GET_VAR_FN.unwrap()(name, vendor, buf) };
}
