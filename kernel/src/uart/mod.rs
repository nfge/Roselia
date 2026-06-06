pub fn serial_print(s:&str){
    use x86_64::instructions::port::Port;
    let mut port: Port<u16> = Port::new(0x3F8);
    for b in s.bytes() {
        unsafe {port.write(b as u16)};
    }

}