#![no_main]
#![no_std]

extern crate alloc;

use acpi_tables::mcfg::McfgEntry;
use x86::io::{inl, outl};

use crate::headers::{PciDevice, PciHeader};
use alloc::{boxed::Box, format, vec::Vec};
use utils::serial_println;

pub mod headers;

pub const PCI_IDS: &[u8] = include_bytes!("pci.ids");

pub unsafe fn read_header(addr: u64) -> PciHeader {
    (addr as *const PciHeader).read_volatile()
}

pub unsafe fn read_header_legacy(
    bus: u8,
    device: u8,
    function: u8,
) -> PciHeader {
    PciHeader {
        vendor_id: read_u16(bus, device, function, 0x00),
        device_id: read_u16(bus, device, function, 0x02),
        command: read_u16(bus, device, function, 0x04),
        status: read_u16(bus, device, function, 0x06),
        revision_id: read_u8(bus, device, function, 0x08),
        prog_if: read_u8(bus, device, function, 0x09),
        subclass: read_u8(bus, device, function, 0x0A),
        class_code: read_u8(bus, device, function, 0x0B),
        cache_line_size: read_u8(bus, device, function, 0x0C),
        latency_timer: read_u8(bus, device, function, 0x0D),
        header_type: read_u8(bus, device, function, 0x0E),
        bist: read_u8(bus, device, function, 0x0F),
    }
}


pub unsafe fn read_u32(bus: u8, device: u8, function: u8, offset: u8) -> u32 {
    let address =
        0x8000_0000u32
        | ((bus as u32) << 16)
        | ((device as u32) << 11)
        | ((function as u32) << 8)
        | ((offset as u32) & 0xFC);

    outl(0xCF8, address);
    inl(0xCFC)
}

pub unsafe fn read_u16(bus: u8, device: u8, function: u8, offset: u8) -> u16 {
    let value = read_u32(bus, device, function, offset & !0x3);

    let shift = ((offset & 0x2) * 8) as u32;
    ((value >> shift) & 0xFFFF) as u16
}

pub unsafe fn read_u8(bus: u8, device: u8, function: u8, offset: u8) -> u8 {
    let value = read_u32(bus, device, function, offset & !0x3);

    let shift = ((offset & 0x3) * 8) as u32;
    ((value >> shift) & 0xFF) as u8
}


pub unsafe fn enumerate_mcfg(entry: &McfgEntry) -> Vec<PciDevice> {
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
pub unsafe fn enumerate_legacy() -> Vec<PciDevice> {
    let mut devices = Vec::new();

    for bus in 0..=255u8 {
        for device in 0..32u8 {
            let value = read_u32(bus, device, 0, 0);

            let vendor_id = (value & 0xFFFF) as u16;
            let device_id = (value >> 16) as u16;

            if vendor_id == 0xFFFF {
                continue;
            }
            let header = read_header_legacy(bus, device, 0);

            devices.push(PciDevice {
                bus,
                device,
                function: 0,
                header,
            });

            if header.header_type & 0x80 != 0 {
                for function in 1..8u8 {
                    let value = read_u32(bus, device, function, 0);

                    let vendor_id = (value & 0xFFFF) as u16;
                    if vendor_id == 0xFFFF {
                        continue;
                    }

                    let header = read_header_legacy(bus, device, function);

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
