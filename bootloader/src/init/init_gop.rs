use uefi::{boot::{get_handle_for_protocol, open_protocol_exclusive}, proto::console::gop::GraphicsOutput, println};

use crate::systable::gop_table::gop_table;


pub fn init_gop() -> gop_table {
    let g_handle = get_handle_for_protocol::<GraphicsOutput>().unwrap();
    println!("Starting kernel and exiting from boot...");
    let mut gop = open_protocol_exclusive::<GraphicsOutput>(g_handle).unwrap();
    let gop_info = gop.current_mode_info();
    let mut fb = gop.frame_buffer();

    let framebuffer = gop_table {
        framebuffer_ptr: fb.as_mut_ptr(),
        size: fb.size(),
        width: gop_info.resolution().0,
        height: gop_info.resolution().1,
        stride: gop_info.stride(),
        mode_info: gop_info,
    };
    return framebuffer;
}