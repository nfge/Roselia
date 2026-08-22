use alloc::vec::Vec;

use crate::ramfs::error::Error;

pub enum NodeData {
    Empty,
    File(Vec<u8>),
    Ops {
        read: Option<fn() -> Vec<u8>>,
        write: Option<fn(&[u8]) -> Result<(), Error>>
    }
}
impl NodeData {
    pub fn virtual_read(f: fn() -> Vec<u8>) -> Self {
        NodeData::Ops { read: Some(f), write: None }
    }
    pub fn control_write(f: fn(&[u8]) -> Result<(), Error>) -> Self {
        NodeData::Ops { read: None, write: Some(f) }
    }
    pub fn ops(read: fn() -> Vec<u8>, write:fn(&[u8]) -> Result<(),Error>) -> Self {
        NodeData::Ops { read: Some(read), write: Some(write) }
    }
}