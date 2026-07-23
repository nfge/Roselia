#![no_main]
#![no_std]

use core::ffi::c_void;

use utils::serial_println;

use crate::{rsdp::Rsdp, sdtheader::SdtHeader, xsdt::Xsdt};

pub mod rsdp;
pub mod sdtheader;

pub mod rsdt;
pub mod xsdt;

pub mod fadt;
pub mod madt;
pub mod mcfg;

pub mod dsdt;

pub mod func;

pub unsafe fn get_table<T>(acpi_table: *const c_void, signature: &[u8; 4]) -> Option<*const T> {
    unsafe {
        let rsdp_ptr = acpi_table as *const Rsdp;
        let rsdp = rsdp_ptr.read_unaligned();

        if &rsdp.signature != b"RSD PTR " {
            serial_println!("Invalid RSDP signature");
            return None;
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

                if &entry_header.signature == signature {
                    return Some(entry_ptr as *const T)
                }
            }
            return None
        } else {
            serial_println!("Root table is not XSDT (found different signature)");
            return None
        }
    }
}
