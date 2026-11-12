use core::{num::NonZeroUsize, sync::atomic::AtomicBool};

use lock_api::{GetThreadId, GuardSend, RawMutex};

use crate::thread::scheduler::Scheduler;


pub mod interruptlock;

pub struct CurrentThreadId;
pub struct ThreadSpinMutex {
    locked: AtomicBool
}

unsafe impl RawMutex for ThreadSpinMutex {
    const INIT: Self = Self {
        locked: AtomicBool::new(false),
    };
    type Guard = GuardSend;
    fn lock(&self) -> Self::Guard {
        while self.locked.compare_exchange_weak(false, true, core::sync::atomic::Ordering::Acquire, core::sync::atomic::Ordering::Relaxed).is_err() {
            core::hint::spin_loop();
        }
        GuardSend::new()
    }
    fn try_lock(&self) -> Option<Self::Guard> {
        match self.locked.compare_exchange_weak(false, true, core::sync::atomic::Ordering::Acquire, core::sync::atomic::Ordering::Relaxed).is_ok() {
            true => Some(GuardSend::new()),
            false => None
        }
    }
    fn is_locked(&self) -> bool {
        self.locked.load(core::sync::atomic::Ordering::Relaxed)
    }
    unsafe fn unlock(&self, _guard: Self::Guard) {
        self.locked.store(false, core::sync::atomic::Ordering::Release);
    }
}

unsafe impl GetThreadId for CurrentThreadId {
    const INIT: Self = Self;
    fn nonzero_thread_id(&self) -> core::num::NonZeroUsize {
        NonZeroUsize::new(Scheduler::current().and_then(|s| s.current_thread()).map_or(usize::MAX, |t| t as usize + 1)).unwrap()
    }
}

