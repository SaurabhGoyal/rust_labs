# Practice projects in Rust.
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

## torrentrs [Not Started Yet]
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

## workflow_engine [Not Started Yet]
- **Description** - Workflow orchestration tool - similar to [dagger](https://github.com/wayfair-incubator/dagger) or [Uber cadence](https://github.com/uber/cadence/tree/master). Refer to [awesome-workflow-engines repo](https://github.com/meirwah/awesome-workflow-engines) for more knowledge.
- **Target concepts** -
  - thread -
  - sync -
  - net -
  - async -
- **Further changes** -

# Reference projects
- Small-Medium projects (<1k stars, <1k commits) - easy to go through and learn from.
  - [bendy](https://github.com/P3KI/bendy) - Bencode (bittorrent metainfo file encoding) encoder/decoder.
  - [console](https://github.com/console-rs/console) - Terminal manipulator for building CLIs.
  - [flamegraph](https://github.com/flamegraph-rs/flamegraph) - Flamegraph profiler for binaries.
  - [sysinfo](https://github.com/GuillaumeGomez/sysinfo) - Cross platform system information (process, memory, disk, network, devices) library.
  - [sig](https://github.com/ynqa/sig/tree/main) - Interactive grep on a stream
  - [ncspot](https://github.com/hrkfdn/ncspot) - Spotify terminal client
- Fairly large projects
  - [hyperswitch](https://github.com/juspay/hyperswitch) - Open Payment Gateway (By Juspay)
  - [rustdesk](https://github.com/rustdesk/rustdesk) - Remote Desktop (Same as Teamviewer)
  - [rqbit](https://github.com/ikatson/rqbit) - Torrent client.
- Also check
  - https://lib.rs/ - index of crates.io
  - [Awesome-rust](https://github.com/rust-unofficial/awesome-rust)
  - [Idiomatic rust index](https://github.com/mre/idiomatic-rust)
  - Github ([search(<100 stars, updated since May, 2024)](https://github.com/search?q=stars%3A%3C100+pushed%3A%3E2024-05-01+language%3ARust&type=Repositories&ref=advsearch&l=Rust&l=) and [trending](https://github.com/trending/rust?since=daily))
