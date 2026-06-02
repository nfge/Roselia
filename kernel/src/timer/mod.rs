// pub mod timer;
pub(in crate::timer) mod sleep;
pub mod irq;
use core::sync::atomic::AtomicU64;

pub use sleep::sleep;

pub static TICKS: AtomicU64 = AtomicU64::new(0);

pub fn time_ms() -> u64 {
    return TICKS.load(core::sync::atomic::Ordering::Relaxed);
}