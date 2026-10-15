use x86_64::{
    PhysAddr, VirtAddr, structures::paging::{
        FrameAllocator, Mapper, OffsetPageTable, Page, PageSize, PageTableFlags, PhysFrame, Size4KiB, mapper::MapToError,
    },
};

use crate::{
    PAGETABLE_POOL_ALLOCATOR, log_err,
    memory::multi_allocator::{alloc_frames, free_frames},
};

const POOL_GROW_PAGES: usize = 256;

pub const POOL_GROW_COST: usize = ((POOL_GROW_PAGES + 512 - 2) / 512 + 1)
    + ((POOL_GROW_PAGES + 512 * 512 - 2) / (512 * 512) + 1)
    + ((POOL_GROW_PAGES + 512 * 512 * 512 - 2) / (512 * 512 * 512) + 1);

const POOL_RESERVE: usize = ((1 + 512 - 2) / 512 + 1)
    + ((1 + 512 * 512 - 2) / (512 * 512) + 1)
    + ((1 + 512 * 512 * 512 - 2) / (512 * 512 * 512) + 1)
    + POOL_GROW_COST;

#[repr(C)]
pub struct Chunk {
    pub start: u64,
    pub count: usize,
    prev: u64,
}

pub struct PoolAllocator {
    offset: u64,
    head: u64,
    next: u64,
    remaining: usize,
    reserved: usize,
}

impl PoolAllocator {
    pub fn new(start: u64, page_count: usize, offset: u64) -> Self {
        let mut pool = Self {
            offset: offset,
            head: 0,
            next: 0,
            remaining: 0,
            reserved: 0,
        };
        pool.add_chunk(start, page_count);
        pool
    }

    fn add_chunk(&mut self, start: u64, count: usize) {
        let chunk = Chunk {
            start: start,
            count: count,
            prev: self.head,
        };
        unsafe {
            ((start + self.offset) as *mut Chunk).write(chunk);
        }
        self.head = start;
        self.next = start + 4096;
        self.remaining = (count - 1) * 4096;
    }
    fn take(&mut self, count: usize) -> u64 {
        let addr = self.next;
        self.next += (count * 4096) as u64;
        self.remaining -= count * 4096;
        addr
    }

    pub fn alloc_frames(&mut self, count: usize) -> Option<u64> {
        if self.remaining < (count + self.reserved) * 4096 {
            return None;
        }
        Some(self.take(count))
    }

    pub fn chunks(&self) -> impl Iterator<Item = Chunk> + '_ {
        let mut c = self.head;
        core::iter::from_fn(move || {
            if c == 0 {
                return None;
            }
            let chunk = unsafe { ((c + self.offset) as *const Chunk).read() };
            c = chunk.prev;
            Some(chunk)
        })
    }
    pub fn free(&self) -> usize {
        self.remaining / 4096
    }
    pub fn set_reserved(&mut self, count: usize) {
        self.reserved = count
    }
}

unsafe impl FrameAllocator<Size4KiB> for PoolAllocator {
    fn allocate_frame(&mut self) -> Option<PhysFrame<Size4KiB>> {
        if self.remaining < 4096 {
            return None;
        }
        let addr = self.take(1);
        Some(PhysFrame::containing_address(PhysAddr::new(addr)))
    }
}

pub fn ensure_pool_reserve(mapper: &mut OffsetPageTable<'static>) -> Result<(), MapToError<Size4KiB>> {
    let pool = unsafe { &mut *core::ptr::addr_of_mut!(PAGETABLE_POOL_ALLOCATOR) }
        .as_mut()
        .unwrap();
    if pool.free() >= POOL_RESERVE {
        return Ok(())
    }
    let Some(start) = alloc_frames(POOL_GROW_PAGES) else {
        return Err(MapToError::FrameAllocationFailed);
    };
    let mut no = [false; POOL_GROW_PAGES];
    for i in 0..POOL_GROW_PAGES {
        let addr = start + (i as u64) * Size4KiB::SIZE;
        let frame = PhysFrame::<Size4KiB>::containing_address(addr);
        let page = Page::<Size4KiB>::containing_address(VirtAddr::new(addr.as_u64()));
        let result = unsafe {
            mapper.map_to(
                page,
                frame,
                PageTableFlags::PRESENT | PageTableFlags::WRITABLE,
                pool,
            )
        };
        match result {
            Ok(f) => {f.flush();no[i] = true;},
            Err(MapToError::PageAlreadyMapped(p)) => {
                if p == frame {}
            }
            Err(e) => {
                log_err!(
                    "Failed to map pool page {}/{}. e: {:#?}",
                    i,
                    POOL_GROW_PAGES,
                    e
                );
                for j in 0..i {
                    if !no[j] {
                        continue;
                    }
                    let a = start + (j as u64) * Size4KiB::SIZE;
                    let p = Page::<Size4KiB>::containing_address(VirtAddr::new(a.as_u64()));
                    if let Ok((_, f)) = mapper.unmap(p) {
                        f.flush();
                    }
                }
                free_frames(start, POOL_GROW_PAGES);
                return Err(e)
            }
        }
    }
    pool.add_chunk(start.as_u64(), POOL_GROW_PAGES);
    Ok(())
}
