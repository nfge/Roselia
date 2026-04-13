use spin::{Mutex};
use x86::apic::{ApicControl, x2apic::X2APIC};

static APIC: Mutex<Option<X2APIC>> = Mutex::new(None);

pub fn init_apic(){
    let mut apic = X2APIC::new();
    apic.attach();
    *APIC.lock() = Some(apic);
}

pub fn send_eoi(){
    let mut apic = APIC.lock();
    if let Some(ref mut apic) = *apic {
        apic.eoi();
    }
}