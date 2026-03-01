use crate::ColorCode;
use crate::vga::buffer::{Buffer,BUFFER_HEIGHT,BUFFER_WIDTH};
use crate::vga::vga_buffer::ScreenChar;
use x86_64::instructions::port::Port;
use core::fmt;
pub struct Writer {
    pub(crate) row: usize,
    pub(crate) col: usize,
    pub(crate) color_code: ColorCode,
    pub(crate) buffer: &'static mut Buffer,
}


impl Writer {
    fn write_byte(&mut self, byte: u8) {
        match byte {
            b'\n' => { 
                self.new_line();
            }
            b'\x08' => {
                self.clear_last_byte();
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
                self.set_cursor(row, col);
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
        self.set_cursor(self.row, self.col);
    }
    fn clear_last_byte(&mut self){
        if self.col != 0 {
            self.col -= 1;
        } else {
            if self.row != 0 {
                self.row -= 1;
                self.col = BUFFER_WIDTH - 1;
            }
        }
        let row = self.row;
        let col = self.col;
        let color_code = self.color_code;
        self.buffer.chars[row][col] = ScreenChar {
            ascii_character: b' ',
            color_code,
        };
        self.set_cursor(row, col);
    }
    fn set_cursor(&mut self, row: usize, col: usize){
        let pos = row * BUFFER_WIDTH + col;
        unsafe {
            let mut port_command:Port<u8> = Port::new(0x3D4);
            let mut port_data: Port<u8> = Port::new(0x3D5);

            port_command.write(0x0F);
            port_data.write((pos & 0xFF) as u8);

            port_command.write(0x0E);
            port_data.write(((pos >> 8) & 0xFF) as u8);

        }
    }
    fn clear_row(&mut self, row: usize) {
        let blank = ScreenChar {
            ascii_character: b' ',
            color_code: self.color_code,
        };
        for col in 0..BUFFER_WIDTH {
            self.buffer.chars[row][col].write(blank);
        }
    }

    pub fn write_string(&mut self, s: &str) {
        for byte in s.bytes() {
            match byte {
                0x20..=0x7e | b'\n' | b'\x08' => self.write_byte(byte),
                _ => self.write_byte(0xfe),
            }

        }
    }
    // pub fn extern_move_cursor(&mut self, row: i8, col: i8) {
    //     if row < 0 {self.row = self.row.saturating_sub(row.abs() as usize); } else { self.row = self.row.saturating_add(row as usize);}
    //     if col < 0 {self.col = self.col.saturating_sub(col.abs() as usize); } else { self.col = self.col.saturating_add(col as usize);}
    //     self.set_cursor(self.row, self.col);
    // }
}


impl fmt::Write for Writer {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_string(s);
        Ok(())
    }    
}