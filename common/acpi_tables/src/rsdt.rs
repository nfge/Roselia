use crate::sdtheader::SdtHeader;

#[repr(C, packed)]
pub struct Rsdt {
    pub header: SdtHeader
}

impl Rsdt {
    pub unsafe fn entries(&self) -> &[u32] {
        let count = (self.header.length as usize - core::mem::size_of::<SdtHeader>()) / core::mem::size_of::<u32>();
        let ptr = unsafe {(self as *const Rsdt as *const u8).add(core::mem::size_of::<SdtHeader>())} as *const u32;
        unsafe {core::slice::from_raw_parts(ptr, count)}
    }
    pub unsafe fn entry(&self, index: usize) -> u32 {
        unsafe {
            let ptr = (self as *const Rsdt as *const u8)
                .add(core::mem::size_of::<SdtHeader>())
                .add(index * core::mem::size_of::<u32>()) as *const u32;
            ptr.read_unaligned()
        }
    }
}