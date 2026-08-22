#![no_main]
#![no_std]

extern crate alloc;


use alloc::vec::Vec;

use core::ffi::c_void;

use utils::serial_println;

use kernel_api::acpi_tables::{rsdp::Rsdp, sdtheader::SdtHeader, xsdt::Xsdt};



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

pub unsafe fn get_tables<T>(
    acpi_table: *const c_void,
    signature: &[u8; 4]
) -> Vec<*const T> {
    let mut tables = Vec::new();

    unsafe {
        let rsdp = (acpi_table as *const Rsdp).read_unaligned();

        if &rsdp.signature != b"RSD PTR " {
            return tables;
        }

        let root_ptr = if rsdp.revision >= 2 {
            rsdp.xsdt_address as usize as *const c_void
        } else {
            rsdp.rsdt_address as usize as *const c_void
        };

        let root_header = (root_ptr as *const SdtHeader).read_unaligned();

        if &root_header.signature != b"XSDT" {
            return tables;
        }

        let xsdt = &*(root_ptr as *const Xsdt);

        for i in 0..xsdt.entry_count() {
            let entry_addr = xsdt.entry(i);
            let entry_ptr = entry_addr as *const c_void;

            let entry_header = (entry_ptr as *const SdtHeader).read_unaligned();

            if &entry_header.signature == signature {
                tables.push(entry_ptr as *const T);
            }
        }
    }

    tables
}