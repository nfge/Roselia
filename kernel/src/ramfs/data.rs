use alloc::vec::Vec;

pub enum NodeData {
    Empty,
    File(Vec<u8>),
    Virtual(fn() -> Vec<u8>)
}