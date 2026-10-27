use core::sync::atomic::{
    AtomicBool,
    Ordering::{Acquire, Relaxed},
};

use lock_api::{GuardNoSend, Mutex, RawMutex};

pub struct RawInterruptSpinLock {
    locked: AtomicBool,
    interrupt: AtomicBool,
}

unsafe impl RawMutex for RawInterruptSpinLock {
    const INIT: Self = Self {
        locked: AtomicBool::new(false),
        interrupt: AtomicBool::new(false),
    };
    type Guard = GuardNoSend;
    fn lock(&self) -> Self::Guard {
        let on = x86_64::instructions::interrupts::are_enabled();
        x86_64::instructions::interrupts::without_interrupts(|| {
            while self
                .locked
                .compare_exchange_weak(false, true, Acquire, Relaxed)
                .is_err()
            {
                core::hint::spin_loop();
            }
            self.interrupt.store(on, Relaxed);
        });
        GuardNoSend::new()
    }
    fn try_lock(&self) -> Option<Self::Guard> {
        let on = x86_64::instructions::interrupts::are_enabled();
        x86_64::instructions::interrupts::disable();
        if self
            .locked
            .compare_exchange_weak(false, true, Acquire, Relaxed)
            .is_ok()
        {
            self.interrupt.store(on, Relaxed);
            Some(GuardNoSend::new())
        } else {
            if on {
                x86_64::instructions::interrupts::enable();
            }
            None
        }
    }
    fn is_locked(&self) -> bool {
        self.locked.load(Relaxed)
    }
    unsafe fn unlock(&self, _guard: Self::Guard) {
        let on = self.locked.load(Relaxed);
        self.locked.store(false, Relaxed);
        if on {x86_64::instructions::interrupts::enable();}
        
    }
}

pub type InterruptSpinLock<T> = Mutex<RawInterruptSpinLock, T>;