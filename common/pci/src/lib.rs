#![no_main]
#![no_std]

extern crate alloc;

use acpi_tables::mcfg::McfgEntry;

use crate::headers::{PciDevice, PciHeader};
use alloc::{boxed::Box, format, vec::Vec};
use utils::serial_println;

pub mod headers;

pub const PCI_IDS: &[u8] = include_bytes!("pci.ids");

pub unsafe fn read_header(addr: u64) -> PciHeader {
    (addr as *const PciHeader).read_volatile()
}

pub unsafe fn enumerate(entry: &McfgEntry) -> Vec<PciDevice> {
    let mut devices = Vec::new();
    for bus in entry.start_bus..=entry.end_bus {
        for device in 0..32 {
            for function in 0..8 {
                let addr = entry.base_address
                    + ((bus as u64) << 20)
                    + ((device as u64) << 15)
                    + ((function as u64) << 12);

                let header = unsafe { read_header(addr) };
                if header.vendor_id != 0xFFFF {
                    devices.push(PciDevice {
                        bus,
                        device,
                        function,
                        header,
                    });
                }
            }
        }
    }
    devices
}

pub fn check(vendor_id: u16, device_id: u16) -> (Option<&'static str>, Option<&'static str>) {
    let text = core::str::from_utf8(PCI_IDS).unwrap();

    let vendor_id = format!("{:04x}", vendor_id);
    let device_id = format!("\t{:04x}", device_id);
    let mut vendor_name: Option<&'static str> = None;
    let mut device_name: Option<&'static str> = None;

    let mut in_vendor = false;

    for line in text.lines() {
        if line.starts_with('#') || line.is_empty() {
            continue;
        }

        if !line.starts_with('\t') {
            in_vendor = line.starts_with(vendor_id.as_str());

            if in_vendor {
                vendor_name = Some(line[vendor_id.len()..].trim());
            }

            continue;
        }

        if in_vendor && line.starts_with(device_id.as_str()) {
            device_name = Some(line[device_id.len()..].trim());
            return (vendor_name, device_name);
        }
    }
    (None, None)
}
