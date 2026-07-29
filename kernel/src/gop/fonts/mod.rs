pub mod font8x16;

// pub static ARIAL: &[u8] = include_bytes!("arial.ttf");


// SeaBIOS VGA 8x16 font
// Source: https://www.seabios.org/
// Derived from the SeaBIOS project.
// Obtained from: https://github.com/spacerace/romtfont
// License: LGPL-3.0-or-later
pub static VGA_FONT: &[u8; 4096] = include_bytes!("seabios8x16.bin");