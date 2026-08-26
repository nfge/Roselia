#[derive(Debug)]
pub enum LoadError {
    EntryNotExecutable,
    InvalidMagic,
    InvalidAbiVersion
}