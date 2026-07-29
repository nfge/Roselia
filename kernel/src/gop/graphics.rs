use crate::gop::{color::Color, fonts::font8x16::FontChar};
use uefi::proto::console::gop::ModeInfo;

pub struct Graphics {
    pub framebuffer_ptr: *mut u8,
    pub mode_info: ModeInfo,
}

impl Graphics {
    pub fn new(fb: *mut u8, mode_info: ModeInfo) -> Self {
        Self {
            framebuffer_ptr: fb,
            mode_info: mode_info,
        }
    }
    pub fn draw_pixel(&mut self, x: usize, y: usize, color: u32) {
        let fb = self.framebuffer_ptr as *mut u32;
        let offset = y * self.mode_info.stride() + x;

        unsafe {
            fb.add(offset).write_volatile(color);
        }
    }
    pub fn draw_char( &mut self,c: char,font: &[u8],offset_x: usize,offset_y: usize,scale: usize,color: Color) {
        let index = c as usize;

        if index >= 256 {
            return;
        }

        let glyph = &font[index * 16..index * 16 + 16];

        let mut min_x = 8;
        let mut max_x = 0;
        for row in glyph {
            for bit in 0..8 {
                if (row >> (7 - bit)) & 1 == 1 {
                    min_x = min_x.min(bit);
                    max_x = max_x.max(bit);
                }
            }
        }
        if min_x == 8 {
            return;
        }

        let glyph_width = max_x - min_x + 1;
        let shift = ((8 - glyph_width) as isize) / 2 - min_x as isize;

        for (y, row) in glyph.iter().enumerate() {
            for x in 0..8 {
                if (row >> (7 - x)) & 1 == 1 {
                    let x_shifted = x as isize + shift;

                    if !(0..8).contains(&x_shifted) {
                        continue;
                    }

                    let x_pos = offset_x + x_shifted as usize * scale;
                    let y_pos = offset_y + y * scale;

                    for dy in 0..scale {
                        for dx in 0..scale {
                            self.draw_pixel(x_pos + dx, y_pos + dy, color as u32);
                        }
                    }
                }
            }
        }
    }
    pub fn flush(&mut self) {
        for y in 0..self.mode_info.resolution().1 {
            for x in 0..self.mode_info.resolution().0 {
                self.draw_pixel(x, y, Color::Black as u32);
            }
        }
    }
}
