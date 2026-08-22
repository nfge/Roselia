
use acpi_tables::mcfg::McfgEntry;
use alloc::vec::Vec;

use crate::{headers::PciDevice, read::{read_header, read_header_legacy, read_u8, read_u16, read_u32}};

pub unsafe fn enumerate_mcfg(entry: &McfgEntry) -> Vec<PciDevice> {
    let mut devices = Vec::new();

    for bus in entry.start_bus..=entry.end_bus {
        for device in 0..32 {
            let addr = entry.base_address
                + ((bus as u64) << 20)
                + ((device as u64) << 15)
                + ((0u64) << 12);

            let header = unsafe { read_header(addr) };

            if header.vendor_id == 0xFFFF {
                continue;
            }

            devices.push(PciDevice {
                bus,
                device,
                function: 0,
                header,
            });

            if header.header_type & 0x80 != 0 {
                for function in 1..8 {
                    let addr = entry.base_address
                        + ((bus as u64) << 20)
                        + ((device as u64) << 15)
                        + ((function as u64) << 12);

                    let header = unsafe { read_header(addr) };

                    if header.vendor_id == 0xFFFF {
                        continue;
                    }

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