use raw_cpuid::CpuId;

pub fn hardware_random() -> Option<u64> {
    let mut value: u64;
    let mut success: u8;
    let cpuid = CpuId::new();
    if cpuid.get_feature_info().unwrap().has_rdrand() {
        unsafe {
            core::arch::asm!(
                "rdrand {}",
                "setc {}",
                out(reg) value,
                out(reg_byte) success,
                options(nostack)
            ) 
        }
        if success != 0 {
            return Some(value)
        }
    }

    None
}
