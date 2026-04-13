use x86::apic::{ApicControl, x2apic::X2APIC};

static mut APIC: Option<X2APIC> = None;

pub fn init_apic(){
    let mut apic = X2APIC::new();
    apic.attach();
    unsafe {
        APIC = Some(apic);
    }
}

pub fn send_eoi(){
    unsafe {
        let apic_ptr = &raw mut APIC;
        if let Some(apic) = (*apic_ptr).as_mut() {
            apic.eoi();
        }
    }
}