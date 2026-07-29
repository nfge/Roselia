// mod file;
mod error;
mod node;
mod types;

use alloc::{boxed::Box, string::String, vec::Vec};
use error::Error;

use crate::{
    RAMFS,
    ramfs::{
        node::{Node, NodeId},
        types::NodeType,
    },
};

pub struct RamFs {
    nodes: Vec<Node>,
    root: NodeId,
}

impl RamFs {
    pub fn new() -> Self {
        let mut nodes = Vec::new();
        nodes.push(Node::new("/", types::NodeType::Directory, None));
        Self {
            nodes: nodes,
            root: 0,
        }
    }
    pub fn create(&mut self, path: &str) -> Result<(), Error> {
        let parts: Vec<_> = path.split('/').filter(|s| !s.is_empty()).collect();

        let mut current = self.root;

        for (i, part) in parts.iter().enumerate() {
            let is_last = i == parts.len() - 1;

            let mut found = None;

            let current_node: &Node = &self.nodes[current];

            for &child_id in &current_node.children {
                let child: &Node = &self.nodes[child_id];

                if child.name == *part {
                    if !is_last && child.node_type != NodeType::Directory {
                        return Err(Error::NotDirectory);
                    }
                    if is_last {
                        return Err(Error::AlreadyExists);
                    }
                    found = Some(child_id);
                    break;
                }
            }

            match found {
                Some(id) => {
                    current = id;
                }
                None => {
                    let id = self.nodes.len();

                    let node_type = if is_last {
                        types::NodeType::File
                    } else {
                        types::NodeType::Directory
                    };

                    let node = Node::new(part, node_type, Some(current));

                    self.nodes.push(node);
                    self.nodes[current].children.push(id);

                    current = id;
                }
            }
        }
        Ok(())
    }
    pub fn mkdir(&mut self, path: &str) -> Result<(), Error> {
        let mut current: NodeId = self.root;

        for part in path.split('/') {
            if part.is_empty() {
                continue;
            }

            let mut found = None;

            let current_node = &self.nodes[current];

            for &child_id in &current_node.children {
                let child = &self.nodes[child_id];

                if child.name == part {
                    found = Some(child_id);
                    break;
                }
            }

            match found {
                Some(id) => {
                    current = id;
                }
                None => {
                    let id = self.nodes.len();

                    let node = Node::new(part, types::NodeType::Directory, Some(current));

                    self.nodes.push(node);
                    self.nodes[current].children.push(id);

                    current = id;
                }
            }
        }
        Ok(())
    }
    pub fn open(path: &str) -> Result<(),Error>{
        
        Ok(())
    }
}

pub fn create_file(path: &str) {
    unsafe {
        if !RAMFS.is_null() {
            (*RAMFS).create(path);
        }
    }
}
pub fn mkdir(path: &str) {
    unsafe {
        if !RAMFS.is_null() {
            (*RAMFS).mkdir(path);
        }
    }
}
