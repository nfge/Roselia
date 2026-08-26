use alloc::{boxed::Box, vec::Vec};

use kernel_api::ramfs::error::RamFSError;

pub enum NodeData {
    Empty,
    File(Vec<u8>),
    Ops {
        read: Option<Box<dyn Fn() -> Vec<u8> + Send + Sync>>,
        write: Option<fn(&[u8]) -> Result<(), RamFSError>>
    }
}
impl NodeData {
    pub fn virtual_read<F>(f: F) -> Self
    where F: Fn() -> Vec<u8> + Send + Sync + 'static {
        NodeData::Ops { read: Some(Box::new(f)), write: None }
    }
    pub fn control_write(f: fn(&[u8]) -> Result<(), RamFSError>) -> Self {
        NodeData::Ops { read: None, write: Some(f) }
    }
    pub fn ops<F>(read: F, write:fn(&[u8]) -> Result<(),RamFSError>) -> Self
    where F: Fn() -> Vec<u8> + Send + Sync + 'static {
        NodeData::Ops { read: Some(Box::new(read)), write: Some(write) }
    }
}