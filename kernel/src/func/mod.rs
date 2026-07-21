use core::panic;

use crate::uart::serial_print;
use acpi_tables::{
    dsdt::Dsdt,
    fadt::Fadt,
    func::{SLP_EN, SLP_TYP_SHIFT,find_s5_sleep_type},
    rsdp::Rsdp,
    sdtheader::SdtHeader,
    xsdt::Xsdt,
};
use alloc::format;
use bootinfo::time::KernelTime;
use x86::io::outb;
use x86_64::instructions::hlt;

use crate::{ACPI_TABLE, GET_VAR_FN, SET_VAR_FN, TIME_FN, serial_println};

pub fn reset() -> ! {
    unsafe {
        let rsdp_ptr = ACPI_TABLE.unwrap() as *const Rsdp;
        let rsdp = rsdp_ptr.read_unaligned();

        if &rsdp.signature != b"RSD PTR " {
            serial_println!("Invalid RSDP signature");
            panic!("Reset: Invalid RSDP signature");
        }

        let revision = { rsdp.revision };
        let root_ptr = if revision >= 2 {
            rsdp.xsdt_address as usize as *const core::ffi::c_void
        } else {
            rsdp.rsdt_address as usize as *const core::ffi::c_void
        };

        let root_header = (root_ptr as *const SdtHeader).read_unaligned();

        if &root_header.signature == b"XSDT" {
            let xsdt = &*(root_ptr as *const Xsdt);

            let count = xsdt.entry_count();
            for i in 0..count {
                let entry_addr = xsdt.entry(i);
                let entry_ptr = entry_addr as *const core::ffi::c_void;
                let entry_header = (entry_ptr as *const SdtHeader).read_unaligned();

                if &entry_header.signature == b"FACP" {
                    let fadt = &*(entry_ptr as *const Fadt);
                    match fadt.reset_reg.address_space {
                        0 => {
                            (fadt.reset_reg.address as *mut u8).write_volatile(fadt.reset_value);
                        }
                        1 => {
                            outb(fadt.reset_reg.address as u16, fadt.reset_value);
                        }
                        _ => {
                            panic!("Unsupported Address Space");
                        }
                    }
                }
            }
        } else {
            serial_println!("Root table is not XSDT (found different signature)");
        }
    }
    loop {
        hlt();
    }
}
pub fn s5_soft_off() -> ! {
    use x86_64::instructions::port::Port;
    unsafe {
        let rsdp_ptr = ACPI_TABLE.unwrap() as *const Rsdp;
        let rsdp = rsdp_ptr.read_unaligned();

        if &rsdp.signature != b"RSD PTR " {
            serial_println!("Invalid RSDP signature");
            panic!("Reset: Invalid RSDP signature");
        }

        let revision = { rsdp.revision };
        let root_ptr = if revision >= 2 {
            rsdp.xsdt_address as usize as *const core::ffi::c_void
        } else {
            rsdp.rsdt_address as usize as *const core::ffi::c_void
        };

        let root_header = (root_ptr as *const SdtHeader).read_unaligned();

        if &root_header.signature == b"XSDT" {
            let xsdt = &*(root_ptr as *const Xsdt);

            let count = xsdt.entry_count();
            for i in 0..count {
                let entry_addr = xsdt.entry(i);
                let entry_ptr = entry_addr as *const core::ffi::c_void;
                let entry_header = (entry_ptr as *const SdtHeader).read_unaligned();

                if &entry_header.signature == b"FACP" {
                    let fadt = &*(entry_ptr as *const Fadt);
                    let dsdt_ptr = fadt.x_dsdt as *const Dsdt;
                    let dsdt = dsdt_ptr.read_unaligned();
                    
                    let (slp_typa, slp_typb) = find_s5_sleep_type(dsdt.aml_bytes()).unwrap();
                    let mut a: Port<u16> = Port::new(fadt.pm1a_control_block as u16);
                    a.write(((slp_typa as u16) << SLP_TYP_SHIFT) | SLP_EN);
                    if fadt.pm1b_control_block != 0 {
                        let mut b: Port<u16> = Port::new(fadt.pm1b_control_block as u16);
                        b.write(((slp_typb as u16) << SLP_TYP_SHIFT) | SLP_EN);
                    }
                }
            }
        } else {
            serial_println!("Root table is not XSDT (found different signature)");
        }
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
