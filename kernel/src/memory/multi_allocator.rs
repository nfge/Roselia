use bootinfo::kernelinfo::KernelInfo;
use kernel_api::module::raw::{RawModule, RawModules};
use uefi::{
    boot::MemoryType,
    mem::memory_map::{MemoryMap, MemoryMapOwned},
};
use utils::serial_println;
use x86_64::{
    PhysAddr, VirtAddr,
    structures::paging::{
        FrameAllocator, Mapper, Page, PageSize, PageTableFlags, PhysFrame, Size4KiB,
        mapper::MapToError,
    },
};

use crate::{MAPPER, MULTI_ALLOCATOR, log_err, memory::bitmap::Bitmap};

pub struct MultiAllocator<'a> {
    pub bitmap: Bitmap,
    mmap: &'a MemoryMapOwned,
}

impl<'a> MultiAllocator<'a> {
    pub fn new(mmap: &'a MemoryMapOwned) -> Self {
        let bitmap = Bitmap::new(&mmap);
        Self {
            bitmap: bitmap,
            mmap: mmap,
        }
    }
    pub fn init(&mut self, kernel_info: &KernelInfo, modules: RawModules) {
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

        self.reserve_pages(kernel_info.start_address, kernel_info.pages);
        self.reserve_pages(
            kernel_info.stack_info.stack_ptr.as_ptr() as usize,
            kernel_info.stack_info.stack_pages,
        );
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
        for frame in 0..self.bitmap.total_pages {
            if !self.bitmap.is_set(frame) {
                self.bitmap.set(frame);

                return Some(PhysFrame::containing_address(PhysAddr::new(
                    (frame * 4096) as u64,
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
            let frame = (addr.as_u64() as usize / 4096) + i;
            self.bitmap.clear(frame);
        }
    }

    pub fn alloc_page(&mut self) -> Option<Page> {
        let frame = self.alloc_frame()?;
        let page =
            Page::<Size4KiB>::containing_address(VirtAddr::new(frame.start_address().as_u64()));
        match unsafe {
            MAPPER.lock().as_mut().unwrap().map_to(
                page,
                frame,
                PageTableFlags::PRESENT | PageTableFlags::WRITABLE,
                self,
            )
        } {
            Ok(map) => {
                map.flush();
                return Some(page);
            }
            Err(e) => {
                self.free_frame(frame);
                log_err!("Failed to map: {:#?}", e);
                if cfg!(debug_assertions) {
                    serial_println!("Failed to map: {:#?}", e);
                }
            }
        }
        None
    }
    pub fn free_page(&mut self, page: Page<Size4KiB>) {
        match MAPPER.lock().as_mut().unwrap().unmap(page) {
            Ok((frame, flush)) => {
                flush.flush();
                self.free_frame(frame);
            }
            Err(e) => {
                log_err!("Failed to unmap: {:#?}", e);
                if cfg!(debug_assertions) {
                    serial_println!("Failed to unmap: {:#?}", e);
                }
            }
        }
    }
    pub fn alloc_pages(&mut self, count: usize) -> Option<VirtAddr> {
        let physaddr = self.alloc_frames(count)?;
        let mut maped: usize = 0;
        for i in 0..count {
            let addr = physaddr + (i as u64) * Size4KiB::SIZE;
            let frame = PhysFrame::containing_address(addr);
            let page = Page::<Size4KiB>::containing_address(VirtAddr::new(addr.as_u64()));
            let result = unsafe {
                MAPPER.lock().as_mut().unwrap().map_to(
                    page,
                    frame,
                    PageTableFlags::PRESENT | PageTableFlags::WRITABLE,
                    self,
                )
            };
            match result {
                Ok(f) => {
                    f.flush();
                    maped += 1;
                }
                Err(e) => {
                    log_err!("Failed to map page {}/{}. e: {:#?}", i, count, e);
                    if cfg!(debug_assertions) {
                        serial_println!("Failed to map page {}/{}. e: {:#?}", i, count, e);
                    }
                    for j in 0..maped {
                        let addr = physaddr + (j as u64) * Size4KiB::SIZE;
                        let page =
                            Page::<Size4KiB>::containing_address(VirtAddr::new(addr.as_u64()));
                        if let Ok((_, f)) = MAPPER.lock().as_mut().unwrap().unmap(page) {
                            f.flush();
                        }
                    }
                    for j in 0..count {
                        let addr = physaddr + (j as u64) * Size4KiB::SIZE;
                        self.free_frame(PhysFrame::containing_address(addr));
                    }
                    return None;
                }
            }
        }
        Some(VirtAddr::new(physaddr.as_u64()))
    }
    pub fn free_pages(&mut self, addr: VirtAddr, count: usize) {
        for i in 0..count {
            match MAPPER
                .lock()
                .as_mut()
                .unwrap()
                .unmap(Page::<Size4KiB>::containing_address(
                    addr + (i as u64) * Size4KiB::SIZE,
                )) {
                Ok((phys, f)) => {
                    f.flush();
                    self.free_frame(phys);
                }
                Err(e) => {
                    log_err!("Failed to unmap {}/{}. e: {:#?}", i, count, e);
                    if cfg!(debug_assertions) {
                        serial_println!("Failed to unmap {}/{}. e: {:#?}", i, count, e);
                    }
                }
            }
        }
    }
    pub fn map(&mut self, addr: PhysAddr, count: usize) -> Result<(), MapToError<Size4KiB>> {
        let saddr = addr.as_u64();
        for p in 0..count {
            let addr = saddr as usize + p * Size4KiB::SIZE as usize;
            let frame = PhysFrame::<Size4KiB>::containing_address(PhysAddr::new(addr as u64));
            let page = Page::<Size4KiB>::containing_address(VirtAddr::new(addr as u64));
            match unsafe {
                MAPPER.lock().as_mut().unwrap().map_to(
                    page,
                    frame,
                    PageTableFlags::PRESENT | PageTableFlags::WRITABLE,
                    self,
                )
            } {
                Ok(f) => {
                    f.flush();
                }
                Err(MapToError::PageAlreadyMapped(_)) => {
                    continue;
                }
                Err(e) => {
                    return Err(e);
                }
            }
        }
        Ok(())
    }
}

unsafe impl FrameAllocator<Size4KiB> for MultiAllocator<'_> {
    fn allocate_frame(&mut self) -> Option<PhysFrame<Size4KiB>> {
        self.alloc_frame()
    }
}

#[allow(unused)]
pub fn alloc_frame() -> Option<PhysFrame> {
    unsafe {
        if let Some(allocator) = &mut *core::ptr::addr_of_mut!(MULTI_ALLOCATOR) {
            return Some(allocator.alloc_frame().expect("Failed alloc pages"));
        }
    }
    None
}
#[allow(unused)]
pub fn free_frame(frame: PhysFrame) {
    unsafe {
        if let Some(allocator) = &mut *core::ptr::addr_of_mut!(MULTI_ALLOCATOR) {
            allocator.free_frame(frame);
        }
    }
}
#[allow(unused)]
pub fn alloc_frames(count: usize) -> Option<PhysAddr> {
    unsafe {
        if let Some(allocator) = &mut *core::ptr::addr_of_mut!(MULTI_ALLOCATOR) {
            return Some(allocator.alloc_frames(count).expect("Failed alloc pages"));
        }
    }
    None
}
#[allow(unused)]
pub fn free_frames(addr: PhysAddr, count: usize) {
    unsafe {
        if let Some(allocator) = &mut *core::ptr::addr_of_mut!(MULTI_ALLOCATOR) {
            allocator.free_frames(addr, count);
        }
    }
}

#[allow(unused)]
pub fn alloc_page() -> Option<Page> {
    unsafe {
        if let Some(allocator) = &mut *core::ptr::addr_of_mut!(MULTI_ALLOCATOR) {
            return allocator.alloc_page();
        }
    }
    None
}
#[allow(unused)]
pub fn free_page(page: Page<Size4KiB>) {
    unsafe {
        if let Some(allocator) = &mut *core::ptr::addr_of_mut!(MULTI_ALLOCATOR) {
            allocator.free_page(page);
        }
    }
}
#[allow(unused)]
pub fn alloc_pages(count: usize) -> Option<VirtAddr> {
    unsafe {
        if let Some(allocator) = &mut *core::ptr::addr_of_mut!(MULTI_ALLOCATOR) {
            return allocator.alloc_pages(count);
        }
    }
    None
}
#[allow(unused)]
pub fn free_pages(addr: VirtAddr, count: usize) {
    unsafe {
        if let Some(allocator) = &mut *core::ptr::addr_of_mut!(MULTI_ALLOCATOR) {
            allocator.free_pages(addr, count);
        }
    }
}
#[allow(unused)]
pub fn map(addr: PhysAddr, count: usize) -> Result<(), MapToError<Size4KiB>> {
    unsafe {
        if let Some(allocator) = &mut *core::ptr::addr_of_mut!(MULTI_ALLOCATOR) {
            allocator.map(addr, count)?;
            return Ok(());
        } else {
            panic!("Allocator not initialized");
        }
    }
}

#[allow(dead_code)]
pub fn get_total_memory() -> usize {
    let mut total = 0;
    unsafe {
        if let Some(alloc) = &mut *core::ptr::addr_of_mut!(MULTI_ALLOCATOR) {
            total += alloc.bitmap.total_pages
        }
    }
    total * 4
}
#[allow(unused)]
pub fn get_free_mem() -> usize {
    let mut free = 0;
    unsafe {
        if let Some(allocator) = &mut *core::ptr::addr_of_mut!(MULTI_ALLOCATOR) {
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
        if let Some(allocator) = &mut *core::ptr::addr_of_mut!(MULTI_ALLOCATOR) {
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
