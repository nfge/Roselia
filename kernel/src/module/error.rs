use crate::linker::error::RelocateError;

#[derive(Debug)]
pub enum LoadError {
    EntryNotExecutable,
    InvalidMagic,
    InvalidAbiVersion,
    RelocateError(RelocateError)
}


impl From<RelocateError> for LoadError {
    fn from(value: RelocateError) -> Self {
        LoadError::RelocateError(value)
    }
}
