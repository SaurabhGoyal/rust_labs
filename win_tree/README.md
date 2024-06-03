# Win Tree
A program to get the tree structure information of given directory in a recursive manner. This is same as [Windows `tree` command](https://learn.microsoft.com/en-us/windows-server/administration/windows-commands/tree) hence the name `win_tree` except that this supports json formating, depth control and filtering of files based on pattern matching on names.

# Usecase
This command is useful for scenarios where making a complete backup of complete data is too expensive hence a snapshot of the file structure tree can be taken and backed up so that atleast the names of the files is available and those files can be fetched again oif needed. (I am using this to create snapshot on a daily basis of my external 8 TB drive that holds all my media.)

# Arguments 
- [Mandatory] Path - Must always be the first argument.
- [Optional] Depth - [-d <number>] Controls how deep to go to generate the tree. Note that if there are children of a directory which are not included in the tree due to depth control then `size_in_bytes` for those directories and cascadingly for all their parent directories would be null as reporting them  without evaluating children would be incorrect.
- [Optional] Exclude - [-e <regex_pattern>] Controls which paths to exclude from snapshot.

# How to use
```
# [Install cargo](https://doc.rust-lang.org/cargo/getting-started/installation.html)
- `cargo install win_tree`
- `win_tree <path> -d <depth> -e "<pattern>" > snapshot.json`
```

# Local results
## Single threaded build and build-async
Result on running on my external HDD with cap 8 TB and having data of ~4.2 TB across ~15600 files - Built snapshot in 74.27 seconds.
```
saurabh@Saurabh-Raider:/mnt/d/Saurabh/Personal/rust_labs$ cargo run -p win_tree -- /mnt/f/stuff/ -e "^(?:\..*|doc|debug)" > snapshot.json
warning: virtual workspace defaulting to `resolver = "1"` despite one or more workspace members being on edition 2021 which implies `resolver = "2"`
note: to keep the current resolver, specify `workspace.resolver = "1"` in the workspace root's manifest
note: to use the edition 2021 resolver, specify `workspace.resolver = "2"` in the workspace root's manifest
note: for more details see https://doc.rust-lang.org/cargo/reference/resolver.html#resolver-versions
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.36s
     Running `target/debug/win_tree /mnt/f/stuff/ -e '^(?:\..*|doc|debug)'`
Built in **74.273552703s**
Serialised in **134.912809ms**
```

## Multi threaded build-par using rayon
Result on running on my external HDD with cap 8 TB and having data of ~4.2 TB across ~15600 files - Built snapshot in 17.88 seconds.
```
saurabh@Saurabh-Raider:/mnt/d/Saurabh/Personal/rust_labs$ cargo run -p win_tree -- /mnt/f/stuff/ -e "^(?:\..*|doc|debug)" > snapshot.json
warning: virtual workspace defaulting to `resolver = "1"` despite one or more workspace members being on edition 2021 which implies `resolver = "2"`
note: to keep the current resolver, specify `workspace.resolver = "1"` in the workspace root's manifest
note: to use the edition 2021 resolver, specify `workspace.resolver = "2"` in the workspace root's manifest
note: for more details see https://doc.rust-lang.org/cargo/reference/resolver.html#resolver-versions
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.36s
     Running `target/debug/win_tree /mnt/f/stuff/ -e '^(?:\..*|doc|debug)'`
Built in 17.884196059s
Serialised in 149.379899ms
```