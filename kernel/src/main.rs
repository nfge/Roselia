#![no_std]
#![no_main]

// extern crate alloc;

mod bootinfo;
mod gop;
mod keyboard;
mod terminal;
mod timer;
use core::panic::PanicInfo;

use crate::{
    bootinfo::bootinfo::BootInfo,
    gop::{color::Color, graphics::Graphics},
    terminal::Terminal
};
use uefi::{Error,proto::media::file::{File, FileAttribute}, runtime::{Time, TimeCapabilities}};

static mut RESET_FN: Option<
    fn(reset_type: uefi::runtime::ResetType, status: uefi::Status, data: Option<&[u8]>) -> !,
> = None;
static mut TIME_FN: Option<fn() -> Result<(Time,TimeCapabilities), Error<()>>> = None;

#[unsafe(no_mangle)]
pub extern "sysv64" fn kernel_main(boot_ptr: *const BootInfo) -> ! {
    let info = unsafe { &mut *(boot_ptr as *mut BootInfo) };
    unsafe {
        RESET_FN = Some(info.reset);
        TIME_FN = Some(info.time)
    };
    let mut graphics = Graphics::new(info.framebuffer.framebuffer_ptr, info.framebuffer.mode_info);
    graphics.flush();
    let mut terminal = Terminal::new(graphics, 0, 0, 1);
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
    unsafe { RESET_FN.unwrap()(uefi::runtime::ResetType::COLD, uefi::Status::SUCCESS, None) };
}
