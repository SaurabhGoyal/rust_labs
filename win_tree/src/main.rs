//!
//! # Win Tree
//! A program to get the tree structure information of given directory in a recursive manner. This is same as [Windows `tree` command](https://learn.microsoft.com/en-us/windows-server/administration/windows-commands/tree) hence the name `win_tree` except that this supports json formating, depth control and filtering of files based on pattern matching on names.
//!
//! # Usecase
//! This command is useful for scenarios where making a complete backup of complete data is too expensive hence a snapshot of the file structure tree can be taken and backed up so that atleast the names of the files is available and those files can be fetched again oif needed. (I am using this to create snapshot on a daily basis of my external 8 TB drive that holds all my media.)
//!
//! # Arguments
//! - [Mandatory] Path - Must always be the first argument.
//! - [Optional] Depth - [-d <number>] Controls how deep to go to generate the tree. Note that if there are children of a directory which are not included in the tree due to depth control then `size_in_bytes` for those directories and cascadingly for all their parent directories would be null as reporting them  without evaluating children would be incorrect.
//! - [Optional] Exclude - [-e <regex_pattern>] Controls which paths to exclude from snapshot.
//!
//! # Example
//! This command generates snapshots for non-hidden (not starting with a dot) files and directories upto a depth of 3 and dumps it in snapshot.json.
//! ```win_tree . -d 3 -e "^\..*" > snapshot.json```
//!
//! This command generates snapshots for non-hidden (not starting with a dot) files and directories which are not in `doc` or `debug` directories and dumps it in snapshot.json.
//! ```win_tree . -e "^(?:\..*|doc|debug)" > snapshot.json```

mod cli;
mod tree;
use std::env;

fn main() {
    let config = tree::Config::build_from_args(env::args());
    let root = tree::build(config).unwrap();
    let root = serde_json::to_string_pretty(&root).unwrap();
    println!("{root}");
}
