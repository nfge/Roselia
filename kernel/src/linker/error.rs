#[derive(Debug)]
pub enum RelocateError {
    SymbolNotFound,
    NoPTDYNAMIC,
    NORELDATA
}