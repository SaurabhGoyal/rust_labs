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
//! # Example
//! ```win_tree . -d 3 > snapshot.json```

mod cli;
mod tree;
use cli::parse_command;
use std::env;
use tree::build_tree;

fn main() {
    let args: Vec<String> = env::args().collect();
    let cmd_args = parse_command(&args);
    let root = build_tree(cmd_args).unwrap();
    let root = serde_json::to_string_pretty(&root).unwrap();
    println!("{root}");
}
