use core::{arch::x86_64::_rdtsc, sync::atomic::Ordering};

// use crate::{cpu::{cpuinfo::get_frequency,read_pit}};
use x86_64::{instructions::nop};

use crate::timer::{TICKS, TICKS_PER_SEC};

pub fn sleep(ms: u64) {
    let ticks_per_sec = TICKS_PER_SEC.load(Ordering::Relaxed);
    let target = TICKS.load(Ordering::Relaxed) + (ticks_per_sec * ms) / 1000;

    while TICKS.load(Ordering::Relaxed) < target {
        x86_64::instructions::hlt();
    }
}

pub fn spin_sleep(ms: u64) {
    let ticks_per_sec = TICKS_PER_SEC.load(Ordering::Relaxed);
    let target = TICKS.load(Ordering::Relaxed) + (ticks_per_sec * ms) / 1000;

    while TICKS.load(Ordering::Relaxed) < target {
        core::hint::spin_loop();
    }
}