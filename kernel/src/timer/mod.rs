// pub mod timer;
pub mod irq;
pub mod sleep;
use core::{arch::x86_64::{__rdtscp, _mm_lfence, _rdtsc}, ptr::null_mut, sync::atomic::AtomicU64};

use acpi::get::get_table;
use kernel_api::acpi_tables::fadt::Fadt;
use raw_cpuid::CpuId;
use x86::msr::{
    IA32_X2APIC_CUR_COUNT, IA32_X2APIC_DIV_CONF, IA32_X2APIC_INIT_COUNT, IA32_X2APIC_LVT_TIMER,
    rdmsr, wrmsr,
};

use crate::{ACPI_TABLE, cpu};

pub static TICKS: AtomicU64 = AtomicU64::new(0);
pub static TICKS_PER_SEC: AtomicU64 = AtomicU64::new(0);

pub static TSC_FREQ: AtomicU64 = AtomicU64::new(0);

const ACPI_PM_TIMER_FREQ: u64 = 3_579_545;

// Outdated

// const PIT_FREQ_HZ: u64 = 1_193_182;

// unsafe fn pit_wait_ms(ms: u32) {
//     let count = (PIT_FREQ_HZ * ms as u64 / 1000) as u16;

//     let mut port: Port<u8> = Port::new(0x61);
//     let mut cmd: Port<u8> = Port::new(0x43);
//     let mut chan2: Port<u8> = Port::new(0x42);

//     let tmp = unsafe { port.read() };
//     unsafe { port.write((tmp & 0xFD) | 0x01) };

//     unsafe {
//         cmd.write(0b1011_0000);
//         chan2.write((count & 0xFF) as u8);
//         chan2.write((count >> 8) as u8)
//     };

//     while unsafe {port.read()} & 0x20 == 0 {
//         core::hint::spin_loop();
//     }
// }

unsafe fn read_acpi_pm() -> u32 {
    let fadt = unsafe { &*get_table::<Fadt>(ACPI_TABLE.unwrap(), b"FACP").unwrap() };
    match fadt.x_pm_timer_block.address_space {
        0 => unsafe { return (fadt.x_pm_timer_block.address as *mut u32).read_volatile() as u32 },
        1 => unsafe {
            return x86::io::inl(fadt.x_pm_timer_block.address as u16);
        },
        _ => {
            panic!("Unsupported Address Space")
        }
    }
}

unsafe fn acpi_pm_wait_ms(ms: u32) {
    let fadt = unsafe { &*get_table::<Fadt>(ACPI_TABLE.unwrap(), b"FACP").unwrap() };

    let start = unsafe { read_acpi_pm() };

    let mask = if (fadt.flags & 1 << 8) != 0 {
        0xFFFF_FFFF
    } else {
        0x00FF_FFFF
    };
    while unsafe { read_acpi_pm() }.wrapping_sub(start) & mask < ms {
        core::hint::spin_loop();
    }
}

pub fn calibrate() {
    let thz: u64 = 100;
    let cpuid = CpuId::new();
    let fadt = unsafe { &*get_table::<Fadt>(ACPI_TABLE.unwrap(), b"FACP").unwrap() };
    let mask = if (fadt.flags & 1 << 8) != 0 {
        0xFFFF_FFFF
    } else {
        0x00FF_FFFF
    };
    unsafe {
        wrmsr(IA32_X2APIC_LVT_TIMER, 1 << 16);
        wrmsr(IA32_X2APIC_DIV_CONF, 0x3);
        wrmsr(IA32_X2APIC_INIT_COUNT, 0xFFFF_FFFF);
        let count :u64;
        match cpuid
            .get_advanced_power_mgmt_info()
            .unwrap()
            .has_invariant_tsc()
        {
            true => {
                let mut aux: u32 = 0;
                _mm_lfence();
                let start = __rdtscp(&mut aux);
                _mm_lfence();
                for _ in 0..1_000_000 {
                    if __rdtscp(&mut aux).wrapping_sub(start) >= (TSC_FREQ.load(core::sync::atomic::Ordering::Relaxed) / 100) {
                        break;
                    }
                }
                _mm_lfence();
                count = (0xFFFF_FFFF - (rdmsr(IA32_X2APIC_CUR_COUNT) & 0xFFFF_FFFF)) * 100 / thz;
            }
            false => {
                let start = read_acpi_pm();
                for _ in 0..1_000_000 {
                    if read_acpi_pm().wrapping_sub(start) & mask
                        >= (ACPI_PM_TIMER_FREQ / 100) as u32
                    {
                        break;
                    }
                }
                count = (0xFFFF_FFFF - (rdmsr(IA32_X2APIC_CUR_COUNT) & 0xFFFF_FFFF)) * 100 / thz;
            }
        }

        wrmsr(IA32_X2APIC_LVT_TIMER, (1 << 17) | 0x20);
        wrmsr(
            IA32_X2APIC_INIT_COUNT,
            if count < 1000 { 1_000_000 } else { count },
        );
    }
    TICKS_PER_SEC.store(thz, core::sync::atomic::Ordering::Relaxed);
}

pub fn calibrate_tsc() -> u64 {
    let fadt = unsafe { &*get_table::<Fadt>(ACPI_TABLE.unwrap(), b"FACP").unwrap() };
    let mask = if (fadt.flags & 1 << 8) != 0 {
        0xFFFF_FFFF
    } else {
        0x00FF_FFFF
    };

    let pm_start = unsafe { read_acpi_pm() };
    let tsc_start = unsafe { _rdtsc() };

    let target = ACPI_PM_TIMER_FREQ / 10;
    loop {
        let current = unsafe { read_acpi_pm() };
        let elapsed = current.wrapping_sub(pm_start) & mask;
        if elapsed >= target as u32 {
            break;
        }
        core::hint::spin_loop();
    }
    let tsc_end = unsafe { _rdtsc() };
    let pm_end = unsafe { read_acpi_pm() };

    let pm_delta = pm_end.wrapping_sub(pm_start) & mask;
    let tsc_delta = tsc_end - tsc_start;

    if pm_delta == 0 {
        return 0;
    }

    ((tsc_delta as u128 * ACPI_PM_TIMER_FREQ as u128) / pm_delta as u128) as u64
}
