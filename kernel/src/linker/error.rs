#[derive(Debug)]
pub enum RelocateError {
    SymbolNotFound,
    NoPTDYNAMIC,
    NoDTRELA
}