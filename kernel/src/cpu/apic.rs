use crate::{serial_println, uart::serial_print};
use x86::{
    apic::ioapic::IoApic,
    msr::{IA32_APIC_BASE, IA32_X2APIC_LVT_TIMER, rdmsr, wrmsr},
};

pub fn init_x2apic() {
    unsafe {
        wrmsr(0x80F, 0x1FF);
    }
}
pub fn init_lapic() {
    let old = unsafe { rdmsr(IA32_X2APIC_LVT_TIMER) };
    serial_println!("old = {:#x}", old);

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
        let mut ioapic = IoApic::new(0xFEC00000);
        ioapic.enable(1, rdmsr(0x802) as u8);
    }
}

pub fn send_eoi() {
    unsafe {
        wrmsr(0x80B, 0);
    }
}
