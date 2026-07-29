pub mod font8x16;

// pub static ARIAL: &[u8] = include_bytes!("arial.ttf");


// SeaBIOS VGA 8x16 font
// Originally from fntcol16.zip (c) Joseph Gil — Public Domain.
// Bundled and redistributed by the SeaBIOS project (https://www.seabios.org/),
// which itself is LGPL-3.0-or-later, but this specific font asset carries
// its own public-domain notice in SeaBIOS's src/font.c / vgasrc/vgafonts.c.
// Obtained from: https://github.com/spacerace/romfont
// License: Public Domain
pub static VGA_FONT: &[u8; 4096] = include_bytes!("seabios8x16.bin");