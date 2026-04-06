use x86_64::instructions::port::Port;

pub fn init_pit() {
    let divisor: u16 = 1193;

    unsafe {
        let mut cmd: Port<u8> = Port::new(0x43);
        let mut data: Port<u8> = Port::new(0x40);

        cmd.write(0x36);

        data.write((divisor & 0xFF) as u8);
        data.write((divisor >> 8) as u8);
    }
}

pub fn read_pit() -> u16    {
    unsafe {
        let mut cmd = Port::<u8>::new(0x43);
        let mut data = Port::<u8>::new(0x40);

        cmd.write(0x00);

        let lo: u8 = data.read();
        let hi: u8 = data.read();

        ((hi as u16) << 8) | lo as u16
    }
}
