use core::{num::NonZeroUsize, sync::atomic::AtomicBool};

use lock_api::{GetThreadId, RawMutex};

pub struct CurrentThreadId;
pub struct RawSpinMutex {
    locked: AtomicBool
}

unsafe impl RawMutex for RawSpinMutex {
    const INIT: Self = Self {
        locked: AtomicBool::new(false),
    };
    type GuardMarker = lock_api::GuardSend;
    fn lock(&self) {
        while self.locked.compare_exchange_weak(false, true, core::sync::atomic::Ordering::Acquire, core::sync::atomic::Ordering::Relaxed).is_err() {
            core::hint::spin_loop();
        }
    }
    fn try_lock(&self) -> bool {
        self.locked.compare_exchange_weak(false, true, core::sync::atomic::Ordering::Acquire, core::sync::atomic::Ordering::Relaxed).is_ok()
    }
    fn is_locked(&self) -> bool {
        self.locked.load(core::sync::atomic::Ordering::Relaxed)
    }
    unsafe fn unlock(&self) {
        self.locked.store(false, core::sync::atomic::Ordering::Release);
    }
}

unsafe impl GetThreadId for CurrentThreadId {
    const INIT: Self = Self;
    fn nonzero_thread_id(&self) -> core::num::NonZeroUsize {
        NonZeroUsize::new(1).unwrap()
    }
}