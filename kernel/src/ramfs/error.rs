#[derive(Debug)]
pub enum Error {
    NotFound,
    NotDirectory,
    NotFile,
    AlreadyExists,
    InvalidPath,
    InvalidOffset,
    Null
}