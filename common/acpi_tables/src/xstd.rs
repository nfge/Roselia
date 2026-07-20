use crate::{rsdt::Rsdt, sdtheader::SdtHeader};

#[repr(C, packed)]
#[derive(Clone, Copy,Debug)]
pub struct Xstd {
    pub header: SdtHeader,
}

impl Xstd {
    pub unsafe fn entries(&self) -> &[u64] {
        let count = (self.header.length as usize - core::mem::size_of::<SdtHeader>())
            / core::mem::size_of::<u64>();
        let ptr =
            unsafe { (self as *const Xstd as *const u8).add(core::mem::size_of::<SdtHeader>()) }
                as *const u64;
        unsafe { core::slice::from_raw_parts(ptr, count) }
    }
    pub unsafe fn entry(&self, index: usize) -> u64 {
        unsafe {
            let ptr = (self as *const Xstd as *const u8)
                .add(core::mem::size_of::<SdtHeader>())
                .add(index * core::mem::size_of::<u64>()) as *const u64;
            ptr.read_unaligned()
        }
    }
}
