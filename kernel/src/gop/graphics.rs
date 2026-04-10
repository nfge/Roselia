
use uefi::proto::console::gop::ModeInfo;
use crate::gop::{color::Color, fonts::font8x16::FontChar};

pub struct Graphics {
    pub framebuffer_ptr: *mut u8,
    pub mode_info: ModeInfo,
}

impl Graphics {
    pub fn new(fb:*mut u8, mode_info: ModeInfo) -> Self {
        Self {
            framebuffer_ptr: fb,
            mode_info: mode_info
        }
    }
    pub fn draw_pixel(&mut self, x: usize, y: usize, color: u32) {
        let fb = self.framebuffer_ptr as *mut u32;
        let offset = y * self.mode_info.stride() + x;

        unsafe {
            fb.add(offset).write_volatile(color);
        }
    }
    pub fn draw_char(&mut self, c: char, font: &[FontChar], offset_x:usize,offset_y:usize, scale: usize, color: Color) {
        if let Some(glyph) = font.iter().find(|f| f.ch == c) {
            for (y, row) in glyph.bitmap.iter().enumerate() {
                for x in 0..8 {
                    if (row >> (7 - x)) & 1 == 1 {
                        for dy in 0..scale {
                            for dx in 0..scale {
                                self.draw_pixel(x * scale + offset_x + dx, y * scale + offset_y + dy, color as u32);
                            }
                        }
                    }
                }
            }
        }
    }
    pub fn flush(&mut self){
        for y in 0..self.mode_info.resolution().1 {
            for x in 0..self.mode_info.resolution().0 {
                self.draw_pixel(x, y, Color::Black as u32);
            }
        }
    }
}
