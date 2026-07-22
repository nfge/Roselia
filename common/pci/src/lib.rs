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

    let mut lines = text.lines();
    for line in text.lines() {
        if line.contains("3b0f") {
            serial_println!("{:?}", line.as_bytes());
        }
    }

    // while let Some(line) = lines.next() {
    //     if line.starts_with("#") || line.is_empty() {
    //         continue;
    //     };
    //     if line.starts_with(vendor_id.as_str()) {
    //         vendor_name = Some(&line[vendor_id.len()..].trim());
    //         while let Some(device_line) = lines.next() {
    //             if !device_line.starts_with("\t") {
    //                 break;
    //             };
    //             if device_line.starts_with(device_id.as_str()) {
    //                 device_name = Some(&device_line[device_id.len()..].trim());
    //                 return (vendor_name, device_name);
    //             }
    //         }
    //         return (vendor_name, None);
    //     }
    // }
    (None, None)

    // for line in text.lines() {
    //     if line.is_empty() || line.starts_with('#') {
    //         continue;
    //     }
    //     if !line.starts_with('\t') && !line.starts_with(' ') {
    //         let mut parts = line.split_whitespace();

    //         if let Some(id) = parts.next() {
    //             if id == vendor_id {
    //                 let name = line[id.len()..].trim();
    //                 vendor_name = Some(name);
    //                 continue;
    //             }
    //             if vendor_name.is_some() {
    //                 break;
    //             }
    //         }
    //         if vendor_name.is_some() {
    //             let trimmed = line.trim_start();

    //             let mut parts = trimmed.split_whitespace();

    //             if let Some(id) = parts.next() {
    //                 if id == device_id {
    //                     let name = trimmed[id.len()..].trim();
    //                     return Some((vendor_name, Some(name)));
    //                 }
    //             }
    //         }
    //     }
    // }

    // vendor_name.map(|v| (Some(v), None))
}
