use core::panic;

use acpi_tables::{fadt::Fadt, rsdp::Rsdp, sdtheader::SdtHeader, xsdt::Xsdt};
use bootinfo::time::KernelTime;
use uefi::runtime::{Time, TimeCapabilities};
use x86::io::outb;
use x86_64::instructions::hlt;

use crate::{ACPI_TABLE, GET_VAR_FN, RESET_FN, SET_VAR_FN, TIME_FN, serial_println};

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
                    serial_println!(
                        "space={} addr={:#p} width={} access={} value={:#x}",
                        fadt.reset_reg.address_space,
                        fadt.reset_reg.address as *mut u8,
                        fadt.reset_reg.bit_width,
                        fadt.reset_reg.access_size,
                        fadt.reset_value
                    );
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
