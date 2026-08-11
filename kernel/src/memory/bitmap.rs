use uefi::{
    boot::MemoryType,
    mem::memory_map::{MemoryMap, MemoryMapOwned},
};

pub struct Bitmap {
    pub bitmap_start: usize,
    pub bitmap_pages: usize,
    pub total_pages: usize,
}

impl Bitmap {
    pub fn new(mmap: &MemoryMapOwned) -> Self {
        let mut best_start: Option<*mut u8> = None;
        let mut best_size: usize = 0;
        let mut total_pages = 0;
        for entry in mmap.entries() {
            if entry.ty == MemoryType::CONVENTIONAL || entry.ty == MemoryType::LOADER_DATA || entry.ty == MemoryType::LOADER_CODE {
                let start_page = entry.phys_start as usize / 4096;
                let end_page = start_page + entry.page_count as usize;
                total_pages = total_pages.max(end_page);
                let size = entry.page_count as usize * 4096;

                if size > best_size {
                    best_size = size;
                    best_start = Some(entry.phys_start as *mut u8);
                }
            }
        }
        let bytes = (total_pages + 7) / 8;
        let bitmap_pages = (bytes + 4095) / 4096;

        let start_addr = best_start.expect("No usable memory");

        Self {
            bitmap_start: start_addr as usize,
            bitmap_pages,
            total_pages,
        }
    }
    pub fn set(&mut self, page: usize) {
        let bitmap = self.bitmap_start as *mut u8;
        let byte = page / 8;
        let bit = page % 8;

        unsafe {
            *bitmap.add(byte) |= 1 << bit;
        }
    }
    pub fn clear(&mut self, page: usize) {
        let bitmap = self.bitmap_start as *mut u8;
        let byte = page / 8;
        let bit = page % 8;

        unsafe {
            *bitmap.add(byte) &= !(1 << bit);
        }
    }
    pub fn is_set(&mut self, page: usize) -> bool {
        let bitmap = self.bitmap_start as *mut u8;
        let byte = page / 8;
        let bit = page % 8;

        unsafe { *bitmap.add(byte) & (1 << bit) != 0 }
    }
}
