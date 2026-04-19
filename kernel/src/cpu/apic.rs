use core::cell::UnsafeCell;

use x86::{
    apic::{ApicControl, x2apic::X2APIC},
    msr::{IA32_APIC_BASE, rdmsr, wrmsr},
};
// pub struct X2Apic(UnsafeCell<X2APIC>);
// unsafe impl Sync for X2Apic {}

// static X2APIC: X2Apic = X2Apic(UnsafeCell::new(X2APIC::new()));

// fn x2apic() -> &'static mut X2APIC {
//     unsafe { &mut *X2APIC.0.get() }
// }

pub fn init_x2apic() {
    unsafe {
        wrmsr(0x80F, 0xFF);
        wrmsr(0x832, 1 << 16);
        wrmsr(0x833, 1 << 16)
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
    init_x2apic();
    x86_64::instructions::interrupts::enable();
}

pub fn send_eoi() {
    unsafe {
        wrmsr(0x80B, 0);
    }
}
