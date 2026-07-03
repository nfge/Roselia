mod file;
mod error;

use alloc::{string::String, vec::Vec,boxed::Box};
use error::Error;
use crate::ramfs::file::File;


pub struct RamFs {
    files: Vec<(usize,Box<File>)>,
    count: usize
}

impl RamFs {
    pub fn new() -> Self {
        Self {
            files: Vec::new(),
            count: 0
        }
    }
    pub fn create_file(&mut self,name:String) -> usize {
        let file = Box::new(File::new(name, Vec::new()));
        self.count += 1;
        self.files.push((self.count,file));
        self.count
    }
    pub fn open_file(&mut self, index:&usize) -> Result<*mut File, Error> {
        for (i, file) in &mut self.files {
            if i == index {
                return Ok(file.as_mut() as *mut File)
            }
        }
        Err(Error::NotFound)
    }
}

