use raw_cpuid::{CpuId, ProcessorBrandString, VendorInfo};
use x86::msr::{IA32_THERM_STATUS, rdmsr};
// use x86::msr::IA32_THERM_STATUS;

// pub fn get_frequency() -> Option<u16> {
//     let cpuid = CpuId::new();
//     return Some(cpuid.get_processor_frequency_info().unwrap().processor_base_frequency());
// }
pub fn get_cpu_vendor() -> Option<VendorInfo> {
    let cpuid = CpuId::new();

    return cpuid.get_vendor_info();
}
pub fn get_cpu_brand_name() -> Option<ProcessorBrandString> {
    let cpuid = CpuId::new();

    return cpuid.get_processor_brand_string();
}
pub fn chech_sse_support() -> bool {
    let cpuid = CpuId::new();
    let f = cpuid.get_feature_info().unwrap();

    f.has_sse() && f.has_sse2()
}
pub fn chech_avx_support() -> bool {
    let cpuid = CpuId::new();
    cpuid.get_feature_info().unwrap().has_avx()
}
pub fn get_cpu_therm() -> Option<u8> {
    match get_cpu_vendor().unwrap().as_str() {
        "GenuineIntel" => {
            let therm_status = unsafe { rdmsr(IA32_THERM_STATUS) };
            let tj_value = unsafe { rdmsr(0x1A2) };
            let valid = ((therm_status >> 31) & 1) != 0;
            let tjmax = ((tj_value >> 16) & 0xFF) as u8;
            if valid {
                let digital_readout = ((therm_status >> 16) & 0x7F) as u8;
                let temp = tjmax - digital_readout;
                Some(temp)
            } else {
                None
            }
        }
        "AuthenticAMD" => None,
        _ => None,
    }
}
// fn cpu_apic() -> bool {
//     let cpuid = CpuId::new();
//     return cpuid.get_feature_info().unwrap().has_apic();
// }
// fn cpu_acpi() -> bool {
//     let cpuid = CpuId::new();
//     return cpuid.get_feature_info().unwrap().has_acpi();
// }
// pub fn cpu_features() -> (bool, bool) {
//     return (cpu_apic(), cpu_acpi());
// }

pub fn get_cpu() -> (Option<VendorInfo>, Option<ProcessorBrandString>) {
    return (get_cpu_vendor(), get_cpu_brand_name());
}
