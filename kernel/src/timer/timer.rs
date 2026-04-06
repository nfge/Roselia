use x86_64::instructions::port::Port;

pub static mut TICKS: u64 = 0;

extern "x86-interrupt" fn timer_interrupt(_stack:x86_64::structures::idt::InterruptStackFrame) {
    unsafe {
        TICKS += 1;
    }
    send_eoi();
}

fn send_eoi(){
    unsafe {
        let mut pic: Port<u8> = Port::new(0x20);
        pic.write(0x20);
    }
}

