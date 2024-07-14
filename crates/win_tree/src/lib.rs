/*!
# Win Tree
A program to get the tree structure information of given directory in a recursive manner. This is same as [Windows `tree` command](https://learn.microsoft.com/en-us/windows-server/administration/windows-commands/tree) hence the name `win_tree` except that this supports json formating, depth control and filtering of files based on pattern matching on names.

# Performance
- **[No support for Symlinks]** Program uses `fs::symlink_metadata` which translates to unix's `lstat` command which is ~10-15% faster that `stat` command as the former does not follow symlinks.
- Reducing `lstat` system call to only once per path.
- Using `rayon` to parallely trigger tasks for each path.

# Usecase
This command is useful for scenarios where making a complete backup of complete data is too expensive hence a snapshot of the file structure tree can be taken and backed up so that atleast the names of the files is available and those files can be fetched again oif needed. (I am using this to create snapshot on a daily basis of my external 8 TB drive that holds all my media.)

# Arguments
- [Mandatory] Path - Must always be the first argument.
- [Optional] Depth - [-d <number>] Controls how deep to go to generate the tree. Note that if there are children of a directory which are not included in the tree due to depth control then `size_in_bytes` for those directories and cascadingly for all their parent directories would be null as reporting them  without evaluating children would be incorrect.
- [Optional] Exclude - [-e <regex_pattern>] Controls which paths to exclude from snapshot.
- [Optional] Build Method - [-m <method_name>] Controls which method will be used to build the tree. Following options are there -
  - serial-async - No parallelisation, recursive implementation.
  - par-rayon - Parallellisation with rayon's `par_bridge` on `read_dir` iterator, recursive implementation. **[This gives results fastest]**.

# Example
```
use win_tree::{build, Config, TreeNode};

let source_path = ".";
let tree_root: TreeNode = win_tree::build(win_tree::Config {
    path: source_path,
    depth_check: Some(5),
    exclude_pattern: None,
    build_method: win_tree::BuildMethod::ParallelRayon,
})
.expect("unable to build tree");
```
*/

mod tree;

pub use tree::*;
