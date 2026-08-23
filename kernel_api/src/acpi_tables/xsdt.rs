use crate::acpi_tables::{sdtheader::SdtHeader};

#[repr(C, packed)]
#[derive(Clone, Copy, Debug)]
pub struct Xsdt {
    pub header: SdtHeader,
}

impl Xsdt {
    pub unsafe fn entry(&self, index: usize) -> u64 {
        unsafe {
            let ptr = (self as *const Xsdt as *const u8)
                .add(core::mem::size_of::<SdtHeader>())
                .add(index * core::mem::size_of::<u64>()) as *const u64;
            ptr.read_unaligned()
        }
    }

    pub unsafe fn entry_count(&self) -> usize {
        (self.header.length as usize - core::mem::size_of::<SdtHeader>())
            / core::mem::size_of::<u64>()
    }
}