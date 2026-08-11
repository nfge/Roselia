use uefi::{
    boot::MemoryType,
    mem::memory_map::{MemoryMap, MemoryMapOwned},
};
use x86_64::{PhysAddr, structures::paging::{PhysFrame, Size4KiB}};

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
    pub fn alloc_page(&mut self) -> Option<PhysFrame> {
        for page in 0..self.bitmap.total_pages {
            if !self.bitmap.is_set(page) {
                self.bitmap.set(page);

                return Some(PhysFrame::containing_address(PhysAddr::new((page * 4096) as u64)));
            }
        }
        None
    }
    pub fn free_page(&mut self, page: PhysFrame) {
        let address = page.start_address().as_u64() as usize;
        let page = address / 4096;
        self.bitmap.clear(page);
    }
    pub fn alloc_pages(&mut self, count: usize) -> Option<PhysAddr> {
        let mut free = 0;
        for page in 0..self.bitmap.total_pages {
            if !self.bitmap.is_set(page) {
                free += 1;
                if free == count {
                    let start = page + 1 -count;

                    for p in start..=page {
                        self.bitmap.set(p);
                    }
                    let start_addr = start * 4096;
                    return Some(PhysAddr::new(start_addr as u64))
                }
            }
            else {
                free = 0;
            }
        }
        None
    }
    pub fn free_pages(&mut self, addr: PhysAddr, count: usize) {
        for i in 0..count {
            let page = (addr.as_u64() as usize / 4096) + i;
            self.bitmap.clear(page);
        }
    }
}
