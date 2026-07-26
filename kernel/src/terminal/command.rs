use alloc::{string::String, vec::Vec};

pub struct Command {
    pub name: String,
    pub args: Vec<String>
}

impl Command {
    pub fn new(name:String, args:Vec<String>) -> Self {
        Self {
            name: name,
            args: args
        }
    }
}