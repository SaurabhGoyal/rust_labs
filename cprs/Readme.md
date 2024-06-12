# cprs (File Copier)
A program to copy files from one location to another.
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

# Performance
- Much faster than the linux `cp` command - transfer of ~23k files with tree-depth of ~10 and internal SSD as source and internal HDD as destination.
```
saurabh@Saurabh-Raider:/mnt/d/Saurabh/Personal/rust_labs$ sudo rm -rf /mnt/e/Saurabh/tmp && mkdir -p /mnt/e/Saurabh/tmp && time cp -r local/ros2-windows /mnt/e/Saurabh/tmp
real    5m32.801s
user    0m1.220s
sys     0m20.353s

saurabh@Saurabh-Raider:/mnt/d/Saurabh/Personal/rust_labs$ sudo rm -rf /mnt/e/Saurabh/tmp && mkdir -p /mnt/e/Saurabh/tmp && time target/release/cprs local/ros2-windows /mnt/e/Saurabh/tmp

[====================================================================================================>]
- Files - 22829 / 22829
- Data (KB) - 569757 / 569757 (100.00%)
-------------
Completed `local/ros2-windows/share/examples_rclcpp_minimal_service/hook/pythonscriptspath.ps1`.


real    1m34.575s
user    0m4.616s
sys     1m24.432s
```
- Windows file explorer copy command is taking 2m01s for the same.
