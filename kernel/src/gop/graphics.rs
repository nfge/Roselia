use crate::gop::{color::Color, fonts::font8x16::FontChar};
use alloc::{vec, vec::Vec};
use uefi::proto::console::gop::ModeInfo;

pub struct Graphics {
    pub framebuffer_ptr: *mut u8,
    pub mode_info: ModeInfo,
    pub back_buffer: Vec<u32>,
}

impl Graphics {
    pub fn new(fb: *mut u8, mode_info: ModeInfo) -> Self {
        let stride = mode_info.stride();
        let height = mode_info.resolution().1;
        Self {
            framebuffer_ptr: fb,
            mode_info: mode_info,
            back_buffer: vec![0u32; stride * height],
        }
    }
    pub fn draw_pixel(&mut self, x: usize, y: usize, color: u32) {
        let (width, height) = self.mode_info.resolution();

        if x >= width || y >= height {
            return;
        }
        let offset = y * self.mode_info.stride() + x;

        self.back_buffer[offset] = color;
    }
    pub fn draw_char(
        &mut self,
        c: char,
        font: &[u8],
        offset_x: usize,
        offset_y: usize,
        scale: usize,
        color: Color,
    ) {
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
    pub fn draw_line(&mut self, x1: isize, y1: isize, x2: isize, y2: isize, color: Color) {
        let mut x = x1;
        let mut y = y1;

        let dx = (x2 - x1).abs();
        let dy = -(y2 - y1).abs();

        let sx = if x1 < x2 { 1 } else { -1 };
        let sy = if y1 < y2 { 1 } else { -1 };

        let mut error = dx + dy;
        loop {
            self.draw_pixel(x as usize, y as usize, color as u32);

            if x == x2 && y == y2 {
                break;
            }

            let e2 = 2 * error;

            if e2 >= dy {
                error += dy;
                x += sx;
            }

            if e2 <= dx {
                error += dx;
                y += sy;
            }
        }
    }
    pub fn flush(&mut self) {
        self.back_buffer.fill(Color::Black as u32);
    }
    pub fn present(&mut self) {
        unsafe {
            core::ptr::copy_nonoverlapping(
                self.back_buffer.as_ptr(),
                self.framebuffer_ptr as *mut u32,
                self.mode_info.stride() * self.mode_info.resolution().1,
            );
        }
    }
}
