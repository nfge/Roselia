#![no_main]
#![no_std]

extern crate alloc;

use kernel_api::acpi_tables::mcfg::McfgEntry;
use kernel_api::pci::PciDevice;

use crate::enumerate::{enumerate_legacy, enumerate_mcfg};
use alloc::format;

pub mod enumerate;
pub mod read;

pub const PCI_IDS: &[u8] = include_bytes!("pci.ids");

pub fn find_by_id(mcfg_entry: &McfgEntry, vendor_id: u16, device_id: u16) -> Option<PciDevice> {
    let devices = unsafe { enumerate_mcfg(mcfg_entry) };
    for device in devices {
        if device.header.vendor_id == vendor_id && device.header.device_id == device_id {
            return Some(PciDevice {
                bus: device.bus,
                device: device.device,
                function: device.function,
                header: device.header,
            });
        }
    }
    let devices = unsafe { enumerate_legacy() };
    for device in devices {
        if device.header.vendor_id == vendor_id && device.header.device_id == device_id {
            return Some(PciDevice {
                bus: device.bus,
                device: device.device,
                function: device.function,
                header: device.header,
            });
        }
    }
    None
}
pub fn find_by_class(mcfg_entry: &McfgEntry, class: u8, subclass: u8) -> Option<PciDevice> {
    let devices = unsafe { enumerate_mcfg(mcfg_entry) };
    for device in devices {
        if device.header.class_code == class && device.header.subclass == subclass {
            return Some(PciDevice {
                bus: device.bus,
                device: device.device,
                function: device.function,
                header: device.header,
            });
        }
    }
    let devices = unsafe { enumerate_legacy() };
    for device in devices {
        if device.header.class_code == class && device.header.subclass == subclass {
            return Some(PciDevice {
                bus: device.bus,
                device: device.device,
                function: device.function,
                header: device.header,
            });
        }
    }
    None
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
