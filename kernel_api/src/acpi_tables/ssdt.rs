use crate::acpi_tables::sdtheader::SdtHeader;

#[repr(C, packed)]
#[derive(Clone, Copy, Debug)]
pub struct Ssdt {
    pub header: SdtHeader,
}

impl Ssdt {
    pub unsafe fn aml_bytes(&self) -> &[u8] {
        let base = self as *const Ssdt as *const u8;
        let header_size = core::mem::size_of::<SdtHeader>();

        let length = self.header.length as usize;

        unsafe { core::slice::from_raw_parts(base.add(header_size), length - header_size) }
    }

    pub fn is_valid_checksum(&self, all_bytes: &[u8]) -> bool {
        all_bytes.iter().fold(0u8, |acc, &b| acc.wrapping_add(b)) == 0
    }
}
