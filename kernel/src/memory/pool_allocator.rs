use x86_64::{
    PhysAddr, VirtAddr, structures::paging::{FrameAllocator, PhysFrame, Size4KiB},
};

use crate::memory::multi_allocator::{alloc_frames, map};

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
}

impl PoolAllocator {
    pub fn new(start: u64, page_count: usize, offset: u64) -> Self {
        let mut pool = Self {
            offset: offset,
            head: 0,
            next: 0,
            remaining: 0,
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

    pub fn alloc_frames(&mut self, count: usize) -> Option<u64> {
        if self.remaining < (count * 4096) {
            let c = (count + 1);
            let addr = alloc_frames(c)?.as_u64();
            self.add_chunk(addr, c);

        }
        let addr = self.next;
        self.next += (count * 4096) as u64;
        self.remaining -= count * 4096;

        Some(addr)
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
}

unsafe impl FrameAllocator<Size4KiB> for PoolAllocator {
    fn allocate_frame(&mut self) -> Option<PhysFrame<Size4KiB>> {
        let addr = self.alloc_frames(1).unwrap();
        // map(PhysAddr::new(addr), VirtAddr::new(addr), 1).expect("map failed");
        Some(PhysFrame::containing_address(PhysAddr::new(addr)))
    }
}
