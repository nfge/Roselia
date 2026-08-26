// mod file;
pub mod data;
mod node;
mod types;

use alloc::vec::Vec;
use data::NodeData;
use kernel_api::ramfs::error::RamFSError;

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
        nodes.push(Node::new(
            "/",
            types::NodeType::Directory,
            None,
            NodeData::Empty,
        ));
        Self {
            nodes: nodes,
            root: 0,
        }
    }
    pub fn create_file(&mut self, path: &str, data: NodeData) -> Result<(), RamFSError> {
        let (parent_path, name) = Self::split_path(path)?;

        let parent = self.resolve_path(parent_path)?;

        let parent_node = &self.nodes[parent];

        for &child_id in &parent_node.children {
            if self.nodes[child_id].name == name {
                return Err(RamFSError::AlreadyExists);
            }
        }

        let id = self.nodes.len();
        let node = Node::new(name, NodeType::File, Some(parent), data);

        self.nodes.push(node);
        self.nodes[parent].children.push(id);

        Ok(())
    }
    pub fn mkdir(&mut self, path: &str) -> Result<(), RamFSError> {
        let (parent_path, name) = Self::split_path(path)?;

        let parent = self.resolve_path(parent_path)?;

        let parent_node = &self.nodes[parent];

        for &child_id in &parent_node.children {
            if self.nodes[child_id].name == name {
                return Err(RamFSError::AlreadyExists);
            }
        }

        let id = self.nodes.len();

        let node = Node::new(name, NodeType::Directory, Some(parent), NodeData::Empty);

        self.nodes.push(node);
        self.nodes[parent].children.push(id);

        Ok(())
    }
    pub fn open(&self, path: &str) -> Result<NodeId, RamFSError> {
        let id = self.resolve_path(path)?;
        Ok(id)
    }
    pub fn read(&self, path: &str) -> Result<Vec<u8>, RamFSError> {
        let node_id = self.resolve_path(path)?;
        let node: &Node = &self.nodes[node_id];
        if node.node_type != NodeType::File {
            return Err(RamFSError::NotFile);
        }

        match &node.data {
            NodeData::Empty => Ok(Vec::new()),
            NodeData::File(data) => Ok(data.clone()),
            NodeData::Ops { read: Some(f), .. } => Ok(f()),
            NodeData::Ops { read: None, .. } => Err(RamFSError::NotSupported),
        }
    }
    pub fn write(&mut self, path: &str, offset: usize, data: &[u8]) -> Result<(), RamFSError> {
        let node_id = self.resolve_path(path)?;
        let node: &mut Node = &mut self.nodes[node_id];
        if node.node_type != NodeType::File {
            return Err(RamFSError::NotFile);
        }

        if let NodeData::Ops { write, .. } = &node.data {
            return match write {
                Some(f) => {
                    let f = *f;
                    f(data)
                }
                None => Err(RamFSError::NotSupported),
            };
        }

        if matches!(node.data, NodeData::Empty) {
            node.data = NodeData::File(Vec::new());
        }

        let end = offset.checked_add(data.len()).ok_or(RamFSError::InvalidOffset)?;

        // if let NodeData::File(buf) = &mut node.data {
        //     if buf.len() < end {
        //         buf.resize(end, 0);
        //     }
        //     buf[offset..end].copy_from_slice(data);
        // }
        match &mut node.data {
            NodeData::File(buf) => {
                if buf.len() < end {
                    buf.resize(end, 0);
                }
                buf[offset..end].copy_from_slice(data);
            }
            _ => return Err(RamFSError::NotSupported),
        }

        Ok(())
    }
    pub fn is_valid(&self, path: &str) -> Result<bool, RamFSError> {
        let _ = self.resolve_path(path)?;
        Ok(true)
    }
    fn split_path<'a>(path: &'a str) -> Result<(&'a str, &'a str), RamFSError> {
        let path = path.trim_end_matches('/');

        if path.is_empty() {
            return Err(RamFSError::InvalidPath);
        }

        match path.rsplit_once('/') {
            Some(("", name)) => Ok(("/", name)),
            Some((parent, name)) => Ok((parent, name)),
            None => Err(RamFSError::InvalidPath),
        }
    }
    fn resolve_path(&self, path: &str) -> Result<NodeId, RamFSError> {
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
                        return Err(RamFSError::NotDirectory);
                    }
                    found = Some(child_id);
                    break;
                }
            }
            match found {
                Some(id) => current = id,
                None => return Err(RamFSError::NotFound),
            }
        }
        Ok(current)
    }
    pub fn check_directory(&self, path: &str) -> Result<Vec<&Node>, RamFSError> {
        let nodeid = self.resolve_path(path)?;
        let node: &Node = &self.nodes[nodeid];

        if node.node_type != NodeType::Directory {
            return Err(RamFSError::NotDirectory);
        }

        let childrens: Vec<&Node> = node.children.iter().map(|id| &self.nodes[*id]).collect();

        Ok(childrens)
    }
}

pub fn create_file(path: &str, data: NodeData) -> Result<(), RamFSError> {
    unsafe {
        if !RAMFS.is_null() {
            (*RAMFS).create_file(path, data)?;
        }
    }
    Ok(())
}
pub fn mkdir(path: &str) -> Result<(), RamFSError> {
    unsafe {
        if !RAMFS.is_null() {
            (*RAMFS).mkdir(path)?;
        }
    }
    Ok(())
}
pub fn read_file(path: &str) -> Result<Vec<u8>, RamFSError> {
    unsafe {
        if !RAMFS.is_null() {
            let data = (*RAMFS).read(path)?;
            return Ok(data);
        } else {
            return Err(RamFSError::Null);
        }
    }
}
pub fn write_file(path: &str, offset: usize, data: &[u8]) -> Result<(), RamFSError> {
    unsafe {
        if !RAMFS.is_null() {
            let _ = (*RAMFS).write(path, offset, data)?;
        }
    }
    Ok(())
}

pub fn is_valid(path: &str) -> Result<(), RamFSError> {
    unsafe {
        if !RAMFS.is_null() {
            let _ = (*RAMFS).is_valid(path)?;
        }
    }
    Ok(())
}

pub fn check_directory(path: &str) -> Result<Vec<&Node>, RamFSError> {
    let childrens: Vec<&Node>;
    unsafe {
        if !RAMFS.is_null() {
            childrens = (*RAMFS).check_directory(path)?;
        } else {
            childrens = Vec::new()
        }
    }
    Ok(childrens)
}
