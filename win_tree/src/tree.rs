use futures::executor::block_on;
use rayon::prelude::*;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io;
use std::path::Path;
use std::sync::Arc;
use std::sync::Mutex;

use crate::threadpool::ThreadPool;
use crate::threadpool::ThreadPoolJobSender;

/// Represents a node in a tree structure, used to represent directories and files.
#[derive(Serialize, Deserialize, Debug)]
pub struct TreeNode {
    /// The name of the node.
    pub name: String,
    /// Indicates whether the node represents a file (`true`) or a directory (`false`).
    pub is_file: bool,
    /// The size of the file in bytes, if it's a file. `None` if it's a directory or if the size is not available.
    pub size_in_bytes: Option<u64>,
    /// The children of the node, representing subdirectories and files.
    pub children: Vec<TreeNode>,
}

/// Represents config for building the tree.
#[derive(Debug)]
pub struct Config {
    /// The path to the directory for which to build the tree.
    pub path: String,
    /// Optional depth check to limit the depth of the tree traversal.
    pub depth_check: Option<u32>,
    /// Optional exclude pattern to exclude certain paths from snapshot.
    pub exclude_pattern: Option<String>,
}

/// Builds a tree structure representing the directory structure starting from the specified path.
///
/// # Arguments
///
/// * `config` - Config specifying the path and other parameters such as depth check and exclude pattern.
///
/// # Returns
///
/// A Result containing a TreeNode representing the root of the tree structure, or an io::Error if the operation fails.
pub fn build(config: Config) -> Result<TreeNode, io::Error> {
    let dir = Path::new(&config.path);
    let dir = &dir.canonicalize().unwrap();
    let ep: String;
    // let exclude_pattern = match config.exclude_pattern {
    //     Some(x) => {
    //         ep = x;
    //         Some(&ep[..])
    //     }
    //     None => None,
    // };
    // block_on(_build(
    //     Path::new(dir),
    //     config.depth_check,
    //     exclude_pattern,
    //     0,
    // ))
    // _build_par(Path::new(dir), config.depth_check, exclude_pattern, 0)
    _build_par_with_threadpool(config.path, config.depth_check, config.exclude_pattern)
}

async fn _build(
    dir: &Path,
    depth_check: Option<u32>,
    exclude_pattern: Option<&str>,
    depth: u32,
) -> Result<TreeNode, io::Error> {
    let mut children: Vec<TreeNode> = vec![];
    let mut total_size: Option<u64> = None;
    if dir.is_file() {
        total_size = Some(dir.metadata()?.len());
    }
    if dir.is_dir() && (depth_check.is_none() || depth < depth_check.unwrap()) {
        total_size = Some(0);
        for entry in fs::read_dir(dir)? {
            let entry = entry?.path();
            let entry = entry.as_path();
            if exclude_pattern.is_some()
                && Regex::new(exclude_pattern.unwrap())
                    .unwrap()
                    .is_match(entry.file_name().unwrap().to_str().unwrap())
            {
                continue;
            }
            let entry_node =
                Box::pin(_build(entry, depth_check, exclude_pattern, depth + 1)).await?;
            // Calculate size only if each of the children also has a calculated size.
            total_size = match (total_size, entry_node.size_in_bytes) {
                (Some(curr_size), Some(child_size)) => Some(curr_size + child_size),
                _ => None,
            };
            children.push(entry_node);
        }
    }
    return Ok(TreeNode {
        name: String::from(dir.file_name().unwrap().to_str().unwrap()),
        is_file: dir.is_file(),
        size_in_bytes: total_size,
        children,
    });
}

fn _build_par(
    dir: &Path,
    depth_check: Option<u32>,
    exclude_pattern: Option<&str>,
    depth: u32,
) -> Result<TreeNode, io::Error> {
    let mut node = TreeNode {
        name: String::from(dir.file_name().unwrap().to_str().unwrap()),
        is_file: dir.is_file(),
        size_in_bytes: None,
        children: vec![],
    };
    if dir.is_file() {
        node.size_in_bytes = Some(dir.metadata()?.len());
    } else if dir.is_dir() && (depth_check.is_none() || depth < depth_check.unwrap()) {
        let node_arc = Arc::new(Mutex::new(node));

        fs::read_dir(dir)?
            .par_bridge()
            .filter(|e| e.is_ok())
            .map(|e| e.unwrap().path())
            .filter(|e| {
                exclude_pattern.is_none()
                    || !Regex::new(exclude_pattern.unwrap())
                        .unwrap()
                        .is_match(e.file_name().unwrap().to_str().unwrap())
            })
            .map(|e| (e, Arc::clone(&node_arc)))
            .for_each(move |(e, parent)| {
                let entry_node =
                    _build_par(e.as_path(), depth_check, exclude_pattern, depth + 1).unwrap();
                // Calculate size only if each of the children also has a calculated size.
                {
                    let mut parent = parent.lock().unwrap();
                    parent.size_in_bytes = match (parent.size_in_bytes, entry_node.size_in_bytes) {
                        (Some(curr_size), Some(child_size)) => Some(curr_size + child_size),
                        _ => None,
                    };
                    parent.children.push(entry_node);
                }
            });
        node = Arc::try_unwrap(node_arc)
            .ok()
            .unwrap()
            .into_inner()
            .ok()
            .unwrap();
    }
    Ok(node)
}

fn _build_par_with_threadpool(
    dir: String,
    depth_check: Option<u32>,
    exclude_pattern: Option<String>,
) -> Result<TreeNode, io::Error> {
    let (pool, sender) = ThreadPool::new::<()>(4);
    let sender = Arc::new(sender);
    let depth_check = depth_check.map(Arc::new);
    let exclude_pattern = exclude_pattern.map(Arc::new);
    let result = _build_par_with_threadpool_unit(
        dir,
        0,
        None,
        depth_check,
        exclude_pattern,
        Arc::clone(&sender),
    );
    drop(sender);
    drop(pool);
    if result.is_err() {
        Err(result.err().unwrap())
    } else {
        Ok(Arc::try_unwrap(result.ok().unwrap())
            .ok()
            .unwrap()
            .into_inner()
            .ok()
            .unwrap())
    }
}

fn _build_par_with_threadpool_unit(
    dir: String,
    depth: u32,
    parent: Option<Arc<Mutex<TreeNode>>>,
    depth_check: Option<Arc<u32>>,
    exclude_pattern: Option<Arc<String>>,
    sender: Arc<ThreadPoolJobSender<()>>,
) -> Result<Arc<Mutex<TreeNode>>, io::Error> {
    let dir = Path::new(&dir).canonicalize().unwrap();
    let mut node = TreeNode {
        name: String::from(dir.file_name().unwrap().to_str().unwrap()),
        is_file: dir.is_file(),
        size_in_bytes: None,
        children: vec![],
    };
    if dir.is_file() {
        node.size_in_bytes = Some(dir.metadata()?.len());
    }
    let node_arc = Arc::new(Mutex::new(node));
    if dir.is_dir() && (depth_check.is_none() || depth < *depth_check.clone().unwrap()) {
        fs::read_dir(dir)?
            .filter(|e| e.is_ok())
            .map(|e| e.unwrap().path())
            .filter(|e| {
                exclude_pattern.is_none()
                    || !Regex::new(exclude_pattern.as_ref().unwrap().as_str())
                        .unwrap()
                        .is_match(e.file_name().unwrap().to_str().unwrap())
            })
            .for_each(|e| {
                let child_dir = e.as_path().to_str().unwrap().to_string();
                let child_depth = depth + 1;
                let child_parent_node = Some(node_arc.clone());
                let child_depth_check = depth_check.clone();
                let child_exclude_pattern = exclude_pattern.clone();
                let child_sender = sender.clone();
                sender.add(Box::new(move || {
                    _build_par_with_threadpool_unit(
                        child_dir,
                        child_depth,
                        child_parent_node,
                        child_depth_check,
                        child_exclude_pattern,
                        child_sender,
                    )
                    .unwrap();
                }))
            });
    }
    Ok(node_arc)
}
