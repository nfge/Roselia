use crate::acpi_tables::sdtheader::SdtHeader;

#[repr(C, packed)]
#[derive(Clone, Copy, Debug)]
pub struct Dsdt {
    pub header: SdtHeader,
}

impl Dsdt {
    // pub unsafe fn aml_bytes(&self) -> Option<&[u8]> {
    //     let header_size = core::mem::size_of::<SdtHeader>();
    //     let total_len = self.header.length as usize;
    //     let body_len = total_len.checked_sub(header_size)?;
    //     let ptr = (self as *const Self as *const u8).add(header_size);
    //     Some(core::slice::from_raw_parts(ptr, body_len))
    // }
    pub unsafe fn aml_bytes(&self) -> &[u8] {
        let base = self as *const Dsdt as *const u8;
        let header_size = core::mem::size_of::<SdtHeader>();

        let length = self.header.length as usize;

        unsafe { core::slice::from_raw_parts(base.add(header_size), length - header_size) }
    }

    pub fn is_valid_checksum(&self, all_bytes: &[u8]) -> bool {
        all_bytes.iter().fold(0u8, |acc, &b| acc.wrapping_add(b)) == 0
    }
}
