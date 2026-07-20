use core::ffi::c_void;

use acpi_tables::{
    madt::{Madt, MadtEntry},
    rsdp::Rsdp,
    sdtheader::SdtHeader,
    xsdt::Xsdt,
};
use x86::{
    apic::ioapic::IoApic,
    msr::{IA32_APIC_BASE, IA32_X2APIC_LVT_TIMER, rdmsr, wrmsr},
};

use crate::{ACPI_TABLE, serial_println};

pub fn init_x2apic() {
    unsafe {
        wrmsr(0x80F, 0x1FF);
    }
}
pub fn init_lapic() {
    unsafe {
        wrmsr(IA32_X2APIC_LVT_TIMER, (1 << 17) | 0x20);
        wrmsr(0x83E, 0x3);
        wrmsr(0x838, 1_000_000);
    }
}

pub fn init_apic() {
    x86_64::instructions::interrupts::disable();

    let mut apic_base = unsafe { rdmsr(IA32_APIC_BASE) };

    apic_base |= 1 << 11;

    apic_base |= 1 << 10;

    unsafe {
        wrmsr(IA32_APIC_BASE, apic_base);
    }

    // init_x2apic();
    // init_ioapic();
    // init_lapic();
    // x86_64::instructions::interrupts::enable();
}
pub fn init_ioapic() {
    unsafe {
        let rsdp_ptr = ACPI_TABLE.unwrap() as *const Rsdp;
        let rsdp = rsdp_ptr.read_unaligned();

        if &rsdp.signature != b"RSD PTR " {
            serial_println!("Invalid RSDP signature");
            return;
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

                if &entry_header.signature == b"APIC" {
                    let madt = &*(entry_ptr as *const Madt);
                    for madt_entry in madt.entries() {
                        if let MadtEntry::IoApic(io) = madt_entry {
                            let mut ioapic = IoApic::new({ io.io_apic_address } as usize);
                            ioapic.enable(1, rdmsr(0x802) as u8);
                        }
                    }
                }
            }
        } else {
            serial_println!("Root table is not XSDT (found different signature)");
        }
    }
}
pub fn send_eoi() {
    unsafe {
        wrmsr(0x80B, 0);
    }
}
