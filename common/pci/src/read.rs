use x86::io::{inl, outl};

use crate::headers::PciHeader;

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
