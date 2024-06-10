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
use std::thread;

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
    pub children: Vec<Arc<TreeNode>>,
    /// Parent of the node.
    #[serde(skip_serializing)]
    parent: Option<Arc<Mutex<TreeNode>>>,
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
    /// Every path is queued in a job queue. A pool of n concurrent threads process each job concurrently.
    /// This is currently giving the worst time performance considering that file info read is not a CPU bound
    ///  process and hence would be causing a lot of context switchingamong threads.
    ParallelThreadPool,
}

impl FromStr for BuildMethod {
    fn from_str(method: &str) -> Result<Self, Self::Err> {
        match method {
            "serial-async" => Ok(Self::SerialAsync),
            "par-rayon" => Ok(Self::ParallelRayon),
            "par-tp" => Ok(Self::ParallelThreadPool),
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
        BuildMethod::ParallelThreadPool => {
            _build_par_with_threadpool(config.path, config.depth_check, config.exclude_pattern)
        }
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
            children.push(Arc::new(entry_node));
        }
    }
    return Ok(TreeNode {
        name: String::from(dir.file_name().unwrap().to_str().unwrap()),
        is_file: dir.is_file(),
        size_in_bytes: total_size,
        children,
        parent: None,
    });
}

fn _build_par(
    dir: &Path,
    depth_check: Option<u32>,
    exclude_pattern: Option<&String>,
    depth: u32,
) -> Result<TreeNode, io::Error> {
    let mut node = TreeNode {
        name: String::from(dir.file_name().unwrap().to_str().unwrap()),
        is_file: dir.is_file(),
        size_in_bytes: None,
        children: vec![],
        parent: None,
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

type NodeProcessingResult = Result<Option<TreeNode>, io::Error>;

fn _build_par_with_threadpool(
    dir: String,
    depth_check: Option<u32>,
    exclude_pattern: Option<String>,
) -> Result<TreeNode, io::Error> {
    let (pool, sender, receiver) = ThreadPool::new::<NodeProcessingResult>(4);
    let sender = Arc::new(sender);
    let depth_check = depth_check.map(Arc::new);
    let exclude_pattern = exclude_pattern.map(Arc::new);
    let result_listener = thread::spawn(move || {
        let mut root = None;
        while let Ok(node) = receiver.recv() {
            if let Ok(Some(node)) = node {
                if node.parent.is_none() {
                    root = Some(node);
                }
            }
        }
        root
    });

    let res = _build_par_with_threadpool_unit(
        dir,
        0,
        None,
        depth_check,
        exclude_pattern,
        Arc::clone(&sender),
    )?;
    drop(sender);
    drop(pool);
    match result_listener.join().unwrap() {
        Some(root) => Ok(root),
        None => match res {
            Some(root) => Ok(root),
            None => panic!("no root found"),
        },
    }
}

fn _build_par_with_threadpool_unit(
    dir: String,
    depth: u32,
    parent: Option<Arc<Mutex<TreeNode>>>,
    depth_check: Option<Arc<u32>>,
    exclude_pattern: Option<Arc<String>>,
    sender: Arc<ThreadPoolJobSender<NodeProcessingResult>>,
) -> NodeProcessingResult {
    let dir = Path::new(&dir).canonicalize().unwrap();
    let node = TreeNode {
        name: String::from(dir.file_name().unwrap().to_str().unwrap()),
        is_file: dir.is_file(),
        size_in_bytes: if dir.is_file() {
            Some(dir.metadata()?.len())
        } else {
            Some(0)
        },
        children: vec![],
        parent,
    };
    let mut node_arc_mut_guard = Arc::new(Mutex::new(node));
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
                let child_parent_node = Some(node_arc_mut_guard.clone());
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
                }))
            });
    }
    let mut root: Option<TreeNode> = None;
    while let Ok(temp_node_mut_guard) = Arc::try_unwrap(node_arc_mut_guard) {
        let mut temp_node = temp_node_mut_guard.into_inner().unwrap();
        let temp_parent_arc = temp_node.parent.clone();
        temp_node.parent = None;
        if let Some(temp_parent_arc) = temp_parent_arc {
            let temp_node_arc = Arc::new(temp_node);
            {
                let mut temp_parent_arc = temp_parent_arc.lock().unwrap();
                temp_parent_arc.children.push(temp_node_arc.clone());
                temp_parent_arc.size_in_bytes =
                    match (temp_parent_arc.size_in_bytes, temp_node_arc.size_in_bytes) {
                        (Some(curr_size), Some(child_size)) => Some(curr_size + child_size),
                        _ => None,
                    };
            }
            node_arc_mut_guard = temp_parent_arc.clone();
        } else {
            root = Some(temp_node);
            break;
        }
    }
    Ok(root)
}
