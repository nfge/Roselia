use crate::acpi_tables::sdtheader::SdtHeader;

#[repr(C,packed)]
#[derive(Copy,Clone)]
pub struct Mcfg {
    pub header: SdtHeader,
    pub reserved: u64
}

#[repr(C,packed)]
#[derive(Copy,Clone)]
pub struct McfgEntry {
    pub base_address: u64,
    pub segment_group: u16,
    pub start_bus: u8,
    pub end_bus: u8,
    pub reserved: u32
}

impl Mcfg {
    pub unsafe fn entry(&self,index:usize) -> McfgEntry {
        unsafe {
            let ptr = (self as *const Self as *const u8)
                .add(core::mem::size_of::<Self>())
                .cast::<McfgEntry>()
                .add(index);
                
            ptr.read_unaligned()
        }
    }
    pub unsafe fn entry_count(&self) -> usize {
        (self.header.length as usize - core::mem::size_of::<Self>())
            / core::mem::size_of::<McfgEntry>()
    }
}