use x86_64::registers::control::{Cr0, Cr0Flags, Cr4, Cr4Flags};

use utils::{serial_println, uart::serial_print};

pub fn init_sse() {
    use super::cpuinfo::chech_sse_support;

    if chech_sse_support() {
        unsafe {
            Cr4::update(|cr4| {
                cr4.insert(Cr4Flags::OSFXSR | Cr4Flags::OSXMMEXCPT_ENABLE);
            });
            Cr0::update(|cr0| {
                cr0.remove(Cr0Flags::EMULATE_COPROCESSOR);
                cr0.insert(Cr0Flags::MONITOR_COPROCESSOR);
            });
        }
        serial_println!("SSE init successful");
    } else {
        serial_println!("SSE not supported");
        return;
    }
}
