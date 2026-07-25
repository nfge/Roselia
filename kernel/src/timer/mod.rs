// pub mod timer;
pub(in crate::timer) mod sleep;
pub mod irq;
use core::{arch::x86_64::_rdtsc, sync::atomic::AtomicU64};

pub use sleep::sleep;
use x86_64::instructions::port::Port;

pub static TICKS: AtomicU64 = AtomicU64::new(0);
pub static TICKS_PER_SEC: AtomicU64 = AtomicU64::new(0);

const PIT_FREQ_HZ: u64 = 1_193_182;

unsafe fn pit_wait_ms(ms: u32) {
    let count = (PIT_FREQ_HZ * ms as u64 / 1000) as u16;

    let mut port_61: Port<u8> = Port::new(0x61);
    let mut cmd: Port<u8> = Port::new(0x43);
    let mut chan2: Port<u8> = Port::new(0x42);

    let tmp = port_61.read();
    port_61.write((tmp & 0xFD) | 0x01);

    cmd.write(0b1011_0000);
    chan2.write((count & 0xFF) as u8);
    chan2.write((count >> 8) as u8);

    while port_61.read() & 0x20 == 0 {
        core::hint::spin_loop();
    }
}


pub fn calibrate() {
    const WINDOW_MS: u32 = 50;

    let s = TICKS.load(core::sync::atomic::Ordering::Relaxed);
    unsafe { pit_wait_ms(WINDOW_MS); } 
    let e = TICKS.load(core::sync::atomic::Ordering::Relaxed);

    let delta = e - s;
    let ticks_per_sec = delta as u64 * 1000 / WINDOW_MS as u64;

    TICKS_PER_SEC.store(ticks_per_sec, core::sync::atomic::Ordering::Relaxed);
}