use raw_cpuid::{CpuId, ProcessorBrandString, VendorInfo};
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
// pub fn get_cpu_therm() -> Option<i32> {
//     let therm = unsafe { x86::msr::rdmsr(IA32_THERM_STATUS) };
//     let delta = ((therm >> 16) & 0x7F) as i32;
//     let tjmax = 100;
//     let temp = tjmax - delta;
//     return Some(temp);
// }
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
