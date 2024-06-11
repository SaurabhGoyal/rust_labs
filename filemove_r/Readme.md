# File Move-R(Rust)
A program to move files from one location to another.
- This would expose multiple methods of moving files including sequential reqcursive, sequential async and parallel.
- The preprocessing includes walking the given location and building a tree using [win_tree](https://crates.io/crates/win_tree) lib.
- The actual transfer then includes copying the files an deleting them once done. (Deletion step is currently pending)
- The trnsfer task is written for one given path as a pure function and can be done in async and by multiple threads concurrently.
- Each task produces events for its path.
- Another thread that listens to all these events, aggregates them to a shared state object and publishes both the event and the updated state to the caller. This was done in a dedicated thread so that the actual transfer task has zero shared memory and thus can run without being blocked. 

# Arguments 
All arguments supported by [win_tree](https://crates.io/crates/win_tree) lib.
- [Mandatory] Path - Must always be the first argument.
- [Optional] Depth - [-d <number>] Controls how deep to go to generate the tree. Note that if there are children of a directory which are not included in the tree due to depth control then `size_in_bytes` for those directories and cascadingly for all their parent directories would be null as reporting them  without evaluating children would be incorrect.
- [Optional] Exclude - [-e <regex_pattern>] Controls which paths to exclude from snapshot.
- [Optional] Build Method - [-m <method_name>] Controls which method will be used to build the tree. Following options are there -
  - serial-async - No parallelisation, recursive implementation.
  - par-rayon - Parallellisation with rayon's `par_bridge` on `read_dir` iterator, recursive implementation.
  - par-tp - Parallellisation with a custom written threadpool, pure function implementation.  
