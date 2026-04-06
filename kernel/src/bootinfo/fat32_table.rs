use uefi::{boot::OpenProtocolParams, proto::media::file::Directory};

#[repr(C)]
pub struct FAT32 {
    pub open_volume: Result<Directory, uefi::Error>,
    pub open_params: OpenProtocolParams
}
