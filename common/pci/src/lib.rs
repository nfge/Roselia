#![no_main]
#![no_std]

extern crate alloc;

use acpi_tables::mcfg::McfgEntry;

use crate::headers::{PciDevice, PciHeader};
use alloc::{format, vec::Vec};
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

pub fn check(vendor_id: u16, device_id: u16) -> Option<(Option<&'static str>, Option<&'static str>)> {
    let (mut vendor, mut device): (&str, &str);
    let text = core::str::from_utf8(PCI_IDS).unwrap();
    let mut lines = text.lines();
    while let Some(line) = lines.next() {
        if line.starts_with(format!("{:04x}", vendor_id).as_str()) {
            vendor = &line[6..].trim();
            while let Some(device_line) = lines.next() {
                let trimmed = device_line.trim_start();
                let indent = device_line.len() - trimmed.len();

                if indent == 0 {
                    break; 
                }
                let deeper = trimmed.trim_start();
                if trimmed.len() != deeper.len() {
                    continue;
                }

                if trimmed.starts_with(format!("{:04x}", device_id).as_str()) {
                    device = trimmed[4..].trim();
                    return Some((Some(vendor), Some(device)));
                }
            }
            return Some((Some(vendor), None));
        }
    }
    None
}
