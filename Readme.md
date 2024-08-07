# Practice projects in Rust.
- To run any of them, install rust tool chain - https://www.rust-lang.org/tools/install
- To run a program, run following command with package name as folder name of the crate inside `crates` folder and args for that program. Check the readme inside that folder to know about its arguments -
```
cargo run -p <package_name> -- <args>

Ex.- cargo run -p cprs -- -r source_dir destination_dir
```

# Projects
## win_tree [Improvements pending]
- **Description** - Windows tree command but with json output.
- **Target concepts** -
  - fs - reading file system
  - io - writing files
  - serde - de/serialization
  - regex - pattern matching
  - async - IO bound application (reads hierarchy of directory including all files)
- **Further changes** -
  - clap - better parameter handling

## tiny_shell [Improvements pending]
- **Description** - Terminal shell that takes commands, parses and runs them.
- **Target concepts** -
  - process - process execution
- **Further changes** -
  - clap - better parameter handling

## sudoku_solver [Improvements pending]
- **Description** - Sudoku companion app that solves a given arrangement using human strategies in an event driven manner.
- **Target concepts** -
  - thread - multiple threads handling various evaluations
  - sync -
    - Arc - Sharing game state with all threads. 
    - Mutex - Keeping game state updates synchronised. 
    - channel - Using for data and control sugnals to be passed between game client and game controller. 
- **Further changes** -
  - Rwlock (instead of Mutex) - Game state is read heavy, parallel reads should be allowed and lock should be only during writes.
  - Future (instead of channel) - If the goal is to control the progress of game based on user input, futures can provide the same based on their inherent design of lazy evaluation.
  - More sudoku strategies - Add more strrategies that can solve more puzzles.

## unbricked [Improvements pending]
- **Description** - Simplest implementation of a brick breaker game.
- **Target concepts** -
  - thread - handling both rendering and user input concurrently.
  - sync -
    - Arc - Sharing game state with both threads. 
    - Mutex - Keeping game state updates synchronised.
  - console - TUI framework - rendering and user input.
- **Further changes** -
  - async and tokio - better state management and concurrency
  - Playability - Larger bricks, faster response time to user control.

## cprs [Improvements pending]
- **Description** - Data copier tool for large amounts of data
- **Target concepts** -
  - win_tree - Used to walk and build tree for given path
  - io - Used to perform copying of one path to given location
  - thread - Threads to listen to events from transfer process and update status
  - rayon - parallellisation of transfer logic
- **Further changes** -
  - async - Test if there is any benefit of transfer-async.
  - benchmark - Benchmarking performance against deep and wide trees and ssd and hdd.

## torrentrs [Split and moved to https://gitlab.com/saurabh.2561/torrentrs]
- **Description** - Torrent downloader in terminal
- **Target concepts** -
  - thread -
  - sync -
  - net -
  - async -
  - crossterm -
  - fs -
  - io -
- **Further changes** -

## Async IO lib [Not Started Yet]
- **Description** - Essentially a tokio clone. To learn underlying system calls such as select and epoll and new non-blocking IO construct such as io_uring.
- **Target concepts** -
  - async io -
  - async net -
- **Further changes** -
- **References** -
  - https://rust-lang.github.io/async-book/02_execution/02_future.htmls
  - https://pages.cs.wisc.edu/~remzi/OSTEP/threads-events.pdf
  - https://github.com/Gilnaa/epoll-rs/
  - https://www.zupzup.org/epoll-with-rust/index.html
  - https://blog.pjam.me/posts/select-syscall-in-rust/
  - https://mbinjamil.dev/writings/understanding-async-io/

## workflow_engine [Not Started Yet]
- **Description** - Workflow orchestration tool - similar to [dagger](https://github.com/wayfair-incubator/dagger) or [Uber cadence](https://github.com/uber/cadence/tree/master). Refer to [awesome-workflow-engines repo](https://github.com/meirwah/awesome-workflow-engines) for more knowledge.
- **Target concepts** -
  - thread -
  - sync -
  - net -
  - async -
- **Further changes** -

# References
## Blogs / feeds
- https://this-week-in-rust.org/
- https://kerkour.com/
- https://blog.yoshuawuyts.com/
- https://smallcultfollowing.com/babysteps/

## Projects
- Small-Medium projects (<1k stars, <1k commits) - easy to go through and learn from.
  - [bytes](https://docs.rs/bytes/1.6.0/bytes/struct.Bytes.html) -  Zero cost bytes buffer management library - essentially an `Arc<Vec<u8>>`.
  - [anyhow](https://github.com/dtolnay/anyhow/) - Error wrapper for easier propagation, diplay and debugging.
  - [bendy](https://github.com/P3KI/bendy) - Bencode (bittorrent metainfo file encoding) encoder/decoder.
  - [console](https://github.com/console-rs/console) - Terminal manipulator for building CLIs.
  - [flamegraph](https://github.com/flamegraph-rs/flamegraph) - Flamegraph profiler for binaries.
  - [sysinfo](https://github.com/GuillaumeGomez/sysinfo) - Cross platform system information (process, memory, disk, network, devices) library.
  - [sig](https://github.com/ynqa/sig/tree/main) - Interactive grep on a stream
  - [ncspot](https://github.com/hrkfdn/ncspot) - Spotify terminal client
- Fairly large projects
  - [vector](https://github.com/vectordotdev/vector) - Observability pipeline.
  - [memorysafety](https://github.com/memorysafety) - Infrastructure wide low-level tools built in a memory-safe manner at https://www.memorysafety.org/.
  - [hyperswitch](https://github.com/juspay/hyperswitch) - Open Payment Gateway (By Juspay)
  - [rustdesk](https://github.com/rustdesk/rustdesk) - Remote Desktop (Same as Teamviewer)
  - [rqbit](https://github.com/ikatson/rqbit) - Torrent client.
  - [nix](https://github.com/nix-rust/nix/) - Abstractions for Unix kernel APIs, essentially more refined wrappers on libc functions.
- Also check
  - https://lib.rs/ - index of crates.io
  - [Awesome-rust](https://github.com/rust-unofficial/awesome-rust)
  - [Idiomatic rust index](https://github.com/mre/idiomatic-rust)
  - Github ([search(<100 stars, updated since May, 2024)](https://github.com/search?q=stars%3A%3C100+pushed%3A%3E2024-05-01+language%3ARust&type=Repositories&ref=advsearch&l=Rust&l=) and [trending](https://github.com/trending/rust?since=daily))

## Useful crates
- [dashmap](https://crates.io/crates/dashmap) - Concurrent hashmap.
- [tracing](https://crates.io/crates/tracing) - Tracing of events and spans in async apps.
- [flatdata](https://crates.io/crates/flatdata) - Library to create data structures following a flat data model.