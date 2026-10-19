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
    type GuardMarker = GuardNoSend;
    fn lock(&self) {
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
        })
    }
    fn try_lock(&self) -> bool {
        let on = x86_64::instructions::interrupts::are_enabled();
        x86_64::instructions::interrupts::disable();
        if self
            .locked
            .compare_exchange_weak(false, true, Acquire, Relaxed)
            .is_ok()
        {
            self.interrupt.store(on, Relaxed);
            true
        } else {
            if on {
                x86_64::instructions::interrupts::enable();
            }
            false
        }
    }
    fn is_locked(&self) -> bool {
        self.locked.load(Relaxed)
    }
    unsafe fn unlock(&self) {
        let on =self.locked.load(Relaxed);
        self.locked.store(false, Relaxed);
        if on {x86_64::instructions::interrupts::enable();}
    }
}

pub type InterruptSpinLock<T> = Mutex<RawInterruptSpinLock, T>;