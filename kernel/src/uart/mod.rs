use core::fmt;
use x86_64::instructions::port::Port;

pub struct SerialPort {
    port: Port<u8>,
}

impl SerialPort {
    pub const fn new() -> Self {
        SerialPort {
            port: Port::new(0x3F8),
        }
    }

    #[inline]
    fn write_byte(&mut self, byte: u8) {
        unsafe {
            self.port.write(byte);
        }
    }
}

impl fmt::Write for SerialPort {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for byte in s.bytes() {
            self.write_byte(byte);
        }
        Ok(())
    }
}

pub fn serial_print(s: &str) {
    let mut port: Port<u8> = Port::new(0x3F8);
    for b in s.bytes() {
        unsafe { port.write(b) };
    }
}

#[macro_export]
macro_rules! serial_println {
    ($($arg:tt)*) => {{
        use core::fmt::Write;
        let mut port = $crate::uart::SerialPort::new();
        let _ = write!(port, $($arg)*);
        let _ = write!(port, "\r\n");
    }};
}