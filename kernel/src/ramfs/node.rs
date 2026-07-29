use alloc::{string::{String, ToString}, vec::Vec};

use crate::ramfs::types::NodeType;

pub type NodeId = usize;

pub struct Node {
    pub name: String,
    pub node_type: NodeType,
    pub parent: Option<NodeId>,
    pub data: Vec<u8>,
    pub children: Vec<NodeId>
}

impl Node {
    pub fn new(name:&str, node_type: NodeType, parent: Option<NodeId>) -> Self {
        Self {
            name: name.to_string(),
            node_type: node_type,
            parent: parent,
            data: Vec::new(),
            children: Vec::new()
        }
    }
}