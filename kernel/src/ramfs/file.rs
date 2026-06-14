use alloc::{string::String, vec::Vec};

pub struct File {
    name: String,
    pub data: Vec<u8>
}

impl File {
    pub fn new(name:String, data:Vec<u8>) -> Self{
        Self {
            name: name,
            data: data
        }
    }
}