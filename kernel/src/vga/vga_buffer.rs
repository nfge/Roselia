use crate::vga::colors::ColorCode;



#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
pub struct ScreenChar {
    pub ascii_character: u8,
    pub color_code: ColorCode,
}

impl ScreenChar {
    pub fn read(&self) -> ScreenChar {
        *self
    }
    pub fn write(&mut self, char: ScreenChar){
        *self = char;
    }
}

