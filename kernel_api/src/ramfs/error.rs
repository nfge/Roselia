#[derive(Debug)]
pub enum RamFSError {
    NotFound,
    NotDirectory,
    NotFile,
    AlreadyExists,
    InvalidPath,
    InvalidOffset,
    NotSupported,
    Null
}