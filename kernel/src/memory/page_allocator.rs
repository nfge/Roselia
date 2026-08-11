use uefi::{
    boot::MemoryType,
    mem::memory_map::{MemoryMap, MemoryMapOwned},
};
use x86_64::structures::paging::{PhysFrame, Size4KiB};

use crate::memory::bitmap::Bitmap;

pub struct PageAllocator<'a> {
    bitmap: Bitmap,
    mmap: &'a MemoryMapOwned,
}

impl<'a> PageAllocator<'a> {
    pub fn new(mmap: &'a MemoryMapOwned) -> Self {
        let bitmap = Bitmap::new(&mmap);
        Self {
            bitmap: bitmap,
            mmap: mmap,
        }
    }
    pub fn init(&mut self, kernel_start: usize, kernel_pages: usize) {
        for i in 0..self.bitmap.total_pages {
            self.bitmap.set(i);
        }
        for entry in self.mmap.entries() {
            if entry.ty == MemoryType::CONVENTIONAL {
                let first_page = entry.phys_start / 4096;
                for page in 0..entry.page_count {
                    self.bitmap.clear(first_page as usize + page as usize);
                }
            }
        }
        self.reserve_pages(kernel_start,kernel_pages);
        self.reserve_pages(self.bitmap.bitmap_start, self.bitmap.bitmap_pages);
    }
    pub fn reserve_pages(&mut self, start_addr: usize, pages: usize) {
        let start_page = start_addr / 4096;

        for page in start_page..start_page + pages {
            self.bitmap.set(page);
        }
    }
    // pub fn alloc_page(&mut self) -> PhysFrame {
    //     // PhysFrame::from_start_address::<Size4KiB>(address).unwrap()
    // }
    // pub fn free_page(&mut self, page: PhysFrame) {
    // }
}
