use super::buffer::{BUFFER_HEIGHT,BUFFER_WIDTH,Buffer};
use super::colors::{Color,ColorCode};
use super::screenchar::ScreenChar;
use core::fmt;

pub struct KernelWriter {
    row: usize,
    col: usize,
    pub(crate) color_code: ColorCode,
    pub(crate) buffer: &'static mut Buffer,
}

impl KernelWriter {
    fn write_byte(&mut self, byte: u8) {
        match byte {
            b'\n' => { 
                self.new_line();
            }
            byte => {
                if self.col >= BUFFER_WIDTH {
                    self.new_line();
                }

                let row = self.row;
                let col = self.col;

                let color_code = self.color_code;

                self.buffer.chars[row][col] = ScreenChar {
                    ascii_character: byte,
                    color_code,
                };
                self.col += 1;
            }
        }
    }
    fn new_line(&mut self) {
        for row in 1..BUFFER_HEIGHT {
            for col in 0..BUFFER_WIDTH {
                let character = self.buffer.chars[row][col].read();
                self.buffer.chars[row][col].write(character);
            }
        }
        // self.clear_row(BUFFER_HEIGHT);
        self.row = self.row + 1;
        self.col = 0;
    }
    pub fn write_string(&mut self, s: &str) {
        for byte in s.bytes() {
            match byte {
                0x20..=0x7e | b'\n' | b'\x08' => self.write_byte(byte),
                _ => self.write_byte(0xfe),
            }

        }
    }
}

impl fmt::Write for KernelWriter {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_string(s);
        Ok(())
    }    
}

impl Default for KernelWriter {
    fn default() -> Self {
        Self {
            row: 0,
            col: 0,
            color_code: ColorCode::new(Color::Yellow, Color::Red),
            buffer: unsafe { &mut *(0xb8000 as *mut Buffer) },
        }
    }
}