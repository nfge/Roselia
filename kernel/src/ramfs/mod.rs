mod file;
use alloc::{string::String, vec::Vec,boxed::Box};

use crate::ramfs::file::File;


pub struct RamFs {
    files: Vec<Box<File>>
}

impl RamFs {
    pub fn new() -> Self {
        Self {
            files: Vec::new()
        }
    }
    pub fn create_file(&mut self,name:String) -> (usize,*mut File) {
        let file = Box::new(File::new(name, Vec::new()));
        self.files.push(file);
        let index = self.files.len() - 1;
        (index, self.files.last_mut().unwrap().as_mut() as *mut File)
    }
}

