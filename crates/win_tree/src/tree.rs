use futures::executor::block_on;
use rayon::prelude::*;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io;
use std::path::Path;
use std::str::FromStr;
use std::sync::Arc;
use std::sync::Mutex;

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
    pub children: Vec<Arc<TreeNode>>,
}
/// Represents the method of building the tree. Usually serial-async and parallel-rayon are the most performant.
/// Other methods are added just for benchmarking purposes.
#[derive(Debug)]
pub enum BuildMethod {
    /// Every file path in tree is read sequentially with async await on read call.
    /// This is single threaded with a `block_on` future executor.
    SerialAsync,
    /// Every child path for one directory is read parallelly using rayon's `par_bridge` on iterator.
    /// This is currently giving the best time performance.
    ParallelRayon,
}

impl FromStr for BuildMethod {
    fn from_str(method: &str) -> Result<Self, Self::Err> {
        match method {
            "serial-async" => Ok(Self::SerialAsync),
            "par-rayon" => Ok(Self::ParallelRayon),
            _ => Err(String::from("invalid method")),
        }
    }

    type Err = String;
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
    /// Method of building.
    pub build_method: BuildMethod,
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
    match config.build_method {
        BuildMethod::SerialAsync => block_on(_build(
            Path::new(&config.path),
            config.depth_check,
            config.exclude_pattern.as_ref(),
            0,
        )),
        BuildMethod::ParallelRayon => _build_par(
            Path::new(&config.path),
            config.depth_check,
            config.exclude_pattern.as_ref(),
            0,
        ),
    }
}

async fn _build(
    dir: &Path,
    depth_check: Option<u32>,
    exclude_pattern: Option<&String>,
    depth: u32,
) -> Result<TreeNode, io::Error> {
    let mut children: Vec<Arc<TreeNode>> = vec![];
    let mut total_size: Option<u64> = None;
    let dir_metadata = dir.symlink_metadata()?;
    if dir_metadata.is_file() {
        total_size = Some(dir_metadata.len());
    }
    if dir_metadata.is_dir() && (depth_check.is_none() || depth < depth_check.unwrap()) {
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
            children.push(Arc::new(entry_node));
        }
    }
    return Ok(TreeNode {
        name: String::from(dir.file_name().unwrap().to_str().unwrap()),
        is_file: dir_metadata.is_file(),
        size_in_bytes: total_size,
        children,
    });
}

fn _build_par(
    dir: &Path,
    depth_check: Option<u32>,
    exclude_pattern: Option<&String>,
    depth: u32,
) -> Result<TreeNode, io::Error> {
    let dir_metadata = dir.symlink_metadata()?;
    let mut node = TreeNode {
        name: String::from(dir.file_name().unwrap().to_str().unwrap()),
        is_file: dir_metadata.is_file(),
        size_in_bytes: None,
        children: vec![],
    };
    if dir_metadata.is_file() {
        node.size_in_bytes = Some(dir_metadata.len());
    } else if dir_metadata.is_dir() && (depth_check.is_none() || depth < depth_check.unwrap()) {
        node.size_in_bytes = Some(0);
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
                    parent.children.push(Arc::new(entry_node));
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
