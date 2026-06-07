use core::arch::x86_64::_rdtsc;

// use crate::{cpu::{cpuinfo::get_frequency,read_pit}};
use x86_64::{instructions::nop};

// pub fn sleep_cpuid(ms: u64) {
//     let cycles = get_frequency().unwrap_or(0) * ms / 1000;
//     let start = unsafe { _rdtsc() };
//     while unsafe { _rdtsc() } - start < cycles {
//         nop();
//     }
    
// }


// pub fn sleep(ms:u64){
//     let start = time_ms();

//     while time_ms() - start < ms {
//         nop();
//     } 
// }
pub fn sleep(ms:u64){
    const CYCLES:u64 = 2_100_000;

    let start =unsafe {_rdtsc()};
    let target = start + ms * CYCLES;

    while unsafe { _rdtsc() } < target {
        nop();
    } 
}