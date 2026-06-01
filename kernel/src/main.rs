#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]

mod bootinfo;
mod gop;
mod keyboard;
mod terminal;
mod timer;
mod cpu;
use core::panic::PanicInfo;

use crate::{
    bootinfo::bootinfo::BootInfo,
    gop::{color::Color, graphics::Graphics},
    terminal::Terminal, timer::sleep
};
use uefi::{Error, runtime::{Time, TimeCapabilities}};

static mut RESET_FN: Option<
    fn(reset_type: uefi::runtime::ResetType, status: uefi::Status, data: Option<&[u8]>) -> !,
> = None;
static mut TIME_FN: Option<fn() -> Result<(Time,TimeCapabilities), Error<()>>> = None;

#[unsafe(no_mangle)]
pub extern "sysv64" fn kernel_main(boot_ptr: *const BootInfo) -> ! {
    let info = unsafe { &*boot_ptr };
    unsafe {
        RESET_FN = Some(info.reset);
        TIME_FN = Some(info.time)
    };
    cpu::interrupts::init_idt();
    cpu::pic::disable_pic();
    cpu::apic::init_apic();

    let mut graphics = Graphics::new(info.framebuffer.framebuffer_ptr, info.framebuffer.mode_info);
    graphics.flush();
    let mut terminal = Terminal::new(graphics, 0, 0, 1, Color::White);
    terminal.run();
    loop {
        x86_64::instructions::hlt();
    }
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    let fb = 0x80000000 as *mut u32;
    for i in 0..1000000 {
        unsafe {
            fb.add(i).write_volatile(Color::Red as u32);
        }
    }
    sleep(2000);
    unsafe { RESET_FN.unwrap()(uefi::runtime::ResetType::COLD, uefi::Status::SUCCESS, None) };
}
