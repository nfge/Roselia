// pub mod timer;
pub mod irq;
pub mod sleep;
use core::{arch::x86_64::_rdtsc, sync::atomic::AtomicU64};

use acpi::get::get_table;
use kernel_api::acpi_tables::fadt::Fadt;
use raw_cpuid::CpuId;

use crate::ACPI_TABLE;

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

unsafe fn tsc_wait_ms(ms: u32) {
    let cpuid = CpuId::new();
    let mut freq: u64;
    match cpuid.get_tsc_info() {
        Some(info) => match info.tsc_frequency() {
            Some(f) => freq = f,
            None => {
                freq = TSC_FREQ.load(core::sync::atomic::Ordering::Relaxed);
            }
        },
        None => {
            freq = TSC_FREQ.load(core::sync::atomic::Ordering::Relaxed);
        }
    }

    let count = (freq * ms as u64 / 1000) as u32;

    let start = unsafe { _rdtsc() };

    while unsafe { _rdtsc().wrapping_sub(start) < count as u64 } {
        core::hint::spin_loop();
    }
}

pub fn calibrate() {
    let ms: u32 = 50;

    let s = TICKS.load(core::sync::atomic::Ordering::Relaxed);
    unsafe {
        tsc_wait_ms(ms);
    }
    let e = TICKS.load(core::sync::atomic::Ordering::Relaxed);

    let delta = e - s;
    let ticks_per_sec = delta as u64 * 1000 / ms as u64;

    TICKS_PER_SEC.store(ticks_per_sec, core::sync::atomic::Ordering::Relaxed);
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
