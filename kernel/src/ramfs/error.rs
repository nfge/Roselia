#[derive(Debug)]
pub enum Error {
    NotFound,
    NotDirectory,
    AlreadyExists,
    InvalidPath
}