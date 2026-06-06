#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]

mod gop;
mod keyboard;
mod terminal;
mod timer;
mod cpu;
mod uart;
use core::{panic::PanicInfo};
use bootinfo::BootInfo;
use crate::{
    gop::{color::Color, graphics::Graphics},
    terminal::Terminal, timer::sleep
};
use uart::serial_print;
use uefi::{Error, runtime::{Time, TimeCapabilities}};

static mut FB_PTR: Option<*mut u32> = None;
static mut RESET_FN: Option<
    fn(reset_type: uefi::runtime::ResetType, status: uefi::Status, data: Option<&[u8]>) -> !,
> = None;
static mut TIME_FN: Option<fn() -> Result<(Time,TimeCapabilities), Error<()>>> = None;
static mut SET_VAR_FN: Option<fn(name:&uefi::CStr16,vendor:&uefi::runtime::VariableVendor,attributes:uefi::runtime::VariableAttributes, data: &[u8]) -> Result<(), uefi::Error>> = None;
static mut GET_VAR_FN: Option<for<'buf>fn(name: &uefi::CStr16, vendor: &uefi::runtime::VariableVendor, buf: &'buf mut [u8]) -> Result<(&'buf [u8], uefi::runtime::VariableAttributes), uefi::Error<Option<usize>>>> = None;
#[unsafe(no_mangle)]
pub extern "sysv64" fn kernel_main(boot_ptr: *const BootInfo) -> ! {
    let info = unsafe { &*boot_ptr };
    unsafe {
        FB_PTR = Some(info.gop.framebuffer_ptr.clone() as *mut u32);
        RESET_FN = Some(info.reset);
        TIME_FN = Some(info.time);
        SET_VAR_FN = Some(info.set_var);
        GET_VAR_FN = Some(info.get_var)
    };
    cpu::interrupts::init_idt();
    cpu::pic::disable_pic();
    cpu::apic::init_apic();

    let mut graphics = Graphics::new(info.gop.framebuffer_ptr, info.gop.mode_info);
    graphics.flush();

    let mut terminal = Terminal::new(graphics, 0, 0, 1, Color::White);
    terminal.run();
    loop {
        x86_64::instructions::hlt();
    }
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    unsafe {
        if let Some(fb) = FB_PTR {
            for i in 0..1000000 {
                fb.add(i).write_volatile(Color::Red as u32);
            }
        }
    }
    sleep(3000);
    unsafe { RESET_FN.unwrap()(uefi::runtime::ResetType::COLD, uefi::Status::SUCCESS, None) };
}
