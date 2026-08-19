use core::ptr::null;

use alloc::{string::{String, ToString}, vec::Vec};

use crate::ramfs::{data::NodeData, types::NodeType};

pub type NodeId = usize;

pub struct Node {
    pub name: String,
    pub node_type: NodeType,
    pub parent: Option<NodeId>,
    pub data: NodeData,
    pub children: Vec<NodeId>
}

impl Node {
    pub fn new(name:&str, node_type: NodeType, parent: Option<NodeId>, data_type: NodeData) -> Self {
        Self {
            name: name.to_string(),
            node_type: node_type,
            parent: parent,
            data: data_type,
            children: Vec::new()
        }
    }
}