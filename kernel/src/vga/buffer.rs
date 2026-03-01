use crate::vga::vga_buffer::ScreenChar;

pub const BUFFER_HEIGHT: usize = 20;
pub const BUFFER_WIDTH: usize = 80;

#[repr(transparent)]
pub struct Buffer {
    pub chars: [[ScreenChar; BUFFER_WIDTH]; BUFFER_HEIGHT],
}