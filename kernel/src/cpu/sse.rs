
use x86_64::registers::{control::{Cr0, Cr0Flags, Cr4, Cr4Flags},xcontrol::{XCr0, XCr0Flags}};

use utils::{serial_println};

pub fn init_sse_and_avx() {
    use super::cpuinfo::{chech_sse_support, chech_avx_support};

    if !chech_sse_support() {
        serial_println!("SSE not supported");
        return;
    }

    unsafe {
        Cr4::update(|cr4| {
            cr4.insert(Cr4Flags::OSFXSR | Cr4Flags::OSXMMEXCPT_ENABLE);
            if chech_avx_support() {
                cr4.insert(Cr4Flags::OSXSAVE);
            }
        });

        if chech_avx_support() {
            XCr0::write(XCr0Flags::X87 | XCr0Flags::SSE | XCr0Flags::AVX);
            serial_println!("AVX init successful");
        } else {
            serial_println!("AVX not supported");
        }

        Cr0::update(|cr0| {
            cr0.remove(Cr0Flags::EMULATE_COPROCESSOR);
            cr0.insert(Cr0Flags::MONITOR_COPROCESSOR);
        });
    }
    serial_println!("SSE init successful");
}