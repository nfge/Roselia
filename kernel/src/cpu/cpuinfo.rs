use raw_cpuid::CpuId;


pub fn get_frequency() -> Option<u64> {
    let cpuid = CpuId::new();
    if let Some(cpu_info) = cpuid.get_processor_frequency_info() {
        Some(cpu_info.processor_base_frequency() as u64 * 1_000_000)
    } else {
        None
    }
}

