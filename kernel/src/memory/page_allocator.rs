use core::ops::Add;

use kernel_api::module::raw::{RawModule, RawModules};
use uefi::{
    boot::MemoryType,
    mem::memory_map::{MemoryMap, MemoryMapOwned},
};
use x86_64::{
    PhysAddr,
    structures::paging::{PhysFrame, Size4KiB},
};

use crate::{PAGE_ALLOCATOR, kprintln, memory::bitmap::Bitmap};

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
    pub fn init(&mut self, kernel_start: usize, kernel_pages: usize, modules: RawModules) {
        for i in 0..self.bitmap.total_pages {
            self.bitmap.set(i);
        }
        for entry in self.mmap.entries() {
            if entry.ty == MemoryType::CONVENTIONAL
                || entry.ty == MemoryType::LOADER_DATA
                || entry.ty == MemoryType::LOADER_CODE
            {
                let first_page = entry.phys_start / 4096;
                for page in 0..entry.page_count {
                    self.bitmap.clear(first_page as usize + page as usize);
                }
            }
        }

        self.reserve_pages(kernel_start, kernel_pages);
        self.reserve_pages(self.bitmap.bitmap_start, self.bitmap.bitmap_pages);
        if modules.count != 0 {
            let array_bytes = modules.count * core::mem::size_of::<RawModule>();
            let array_pages = array_bytes.div_ceil(4096);
            self.reserve_pages(modules.ptr as usize, array_pages);

            for i in 0..modules.count {
                let module = unsafe { &*modules.ptr.add(i) };

                let raw_pages = (module.raw_len as usize).div_ceil(4096);
                self.reserve_pages(module.raw_ptr as usize, raw_pages);

                let image_pages = (module.len as usize).div_ceil(4096);
                self.reserve_pages(module.base as usize, image_pages);
            }
        }
    }
    pub fn reserve_pages(&mut self, start_addr: usize, pages: usize) {
        let start_page = start_addr / 4096;

        for page in start_page..start_page + pages {
            self.bitmap.set(page);
        }
    }
    pub fn alloc_frame(&mut self) -> Option<PhysFrame> {
        for page in 0..self.bitmap.total_pages {
            if !self.bitmap.is_set(page) {
                self.bitmap.set(page);

                return Some(PhysFrame::containing_address(PhysAddr::new(
                    (page * 4096) as u64,
                )));
            }
        }
        None
    }
    pub fn free_frame(&mut self, frame: PhysFrame) {
        let address = frame.start_address().as_u64() as usize;
        let frame = address / 4096;
        self.bitmap.clear(frame);
    }
    pub fn alloc_frames(&mut self, count: usize) -> Option<PhysAddr> {
        let mut free = 0;
        for page in 0..self.bitmap.total_pages {
            if !self.bitmap.is_set(page) {
                free += 1;
                if free == count {
                    let start = page + 1 - count;

                    for p in start..=page {
                        self.bitmap.set(p);
                    }
                    let start_addr = start * 4096;
                    return Some(PhysAddr::new(start_addr as u64));
                }
            } else {
                free = 0;
            }
        }
        None
    }
    pub fn free_frames(&mut self, addr: PhysAddr, count: usize) {
        for i in 0..count {
            let page = (addr.as_u64() as usize / 4096) + i;
            self.bitmap.clear(page);
        }
    }
}

#[allow(unused)]
pub fn alloc_frame() -> Option<PhysFrame> {
    unsafe {
        if let Some(allocator) = &mut *core::ptr::addr_of_mut!(PAGE_ALLOCATOR) {
            return Some(allocator.alloc_frame().expect("Failed alloc pages"));
        }
    }
    None
}
#[allow(unused)]
pub fn free_frame(frame: PhysFrame) {
    unsafe {
        if let Some(allocator) = &mut *core::ptr::addr_of_mut!(PAGE_ALLOCATOR) {
            allocator.free_frame(frame);
        }
    }
}
#[allow(unused)]
pub fn alloc_frames(count: usize) -> Option<PhysAddr> {
    unsafe {
        if let Some(allocator) = &mut *core::ptr::addr_of_mut!(PAGE_ALLOCATOR) {
            return Some(allocator.alloc_frames(count).expect("Failed alloc pages"));
        }
    }
    None
}
#[allow(unused)]
pub fn free_frames(addr: PhysAddr, count: usize) {
    unsafe {
        if let Some(allocator) = &mut *core::ptr::addr_of_mut!(PAGE_ALLOCATOR) {
            allocator.free_frames(addr, count);
        }
    }
}

#[allow(dead_code)]
pub fn get_total_memory() -> usize {
    let mut total = 0;
    unsafe {
        if let Some(alloc) = &mut *core::ptr::addr_of_mut!(PAGE_ALLOCATOR) {
            total += alloc.bitmap.total_pages
        }
    }
    total * 4
}
#[allow(unused)]
pub fn get_free_mem() -> usize {
    let mut free = 0;
    unsafe {
        if let Some(allocator) = &mut *core::ptr::addr_of_mut!(PAGE_ALLOCATOR) {
            for entry in allocator.mmap.entries() {
                if entry.ty == MemoryType::CONVENTIONAL {
                    let start_page = entry.phys_start as usize / 4096;

                    for i in 0..entry.page_count as usize {
                        if !allocator.bitmap.is_set(start_page + i) {
                            free += 1;
                        }
                    }
                }
            }
        }
    }

    free * 4
}
#[allow(unused)]
pub fn get_used_mem() -> usize {
    let mut used = 0;

    unsafe {
        if let Some(allocator) = &mut *core::ptr::addr_of_mut!(PAGE_ALLOCATOR) {
            for entry in allocator.mmap.entries() {
                if entry.ty == MemoryType::CONVENTIONAL
                    || entry.ty == MemoryType::LOADER_DATA
                    || entry.ty == MemoryType::LOADER_CODE
                {
                    let start_page = entry.phys_start as usize / 4096;

                    for i in 0..entry.page_count as usize {
                        if allocator.bitmap.is_set(start_page + i) {
                            used += 1;
                        }
                    }
                }
            }
        }
    }

    used * 4
}
