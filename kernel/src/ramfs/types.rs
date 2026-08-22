use alloc::vec::Vec;

#[derive(PartialEq)]
pub enum NodeType {
    File,
    Directory,
    Ops,
}