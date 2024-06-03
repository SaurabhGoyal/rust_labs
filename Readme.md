# Practice projects in Rust.
## win_tree [Improvements pending]
- **Description** - Windows tree command but with json output.
- **Target concepts** -
  - fs - reading file system
  - io - writing files
  - serde - de/serialization
  - regex - pattern matching
- **Further changes** -
  - async - IO bound application (reads hierarchy of directory including all files)
  - clap - better parameter handling

## tiny_shell [Improvements pending]
- **Description** - Terminal shell that takes commands, parses and runs them.
- **Target concepts** -
  - process - process execution
- **Further changes** -
  - async - IO bound application (reads hierarchy of directory including all files)
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

## move_r [Not Started Yet]
- **Description** - Data transfer tool for large amounts of data
- **Target concepts** -
  - thread -
  - sync -
  - async -
  - fs -
  - io -
- **Further changes** -

## torrent_r [Not Started Yet]
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
  - [sysinfo](https://github.com/GuillaumeGomez/sysinfo) - Cross platform system information (process, memory, disk, network, devices) library.
  - [sig](https://github.com/ynqa/sig/tree/main) - Interactive grep on a stream
  - [ncspot](https://github.com/hrkfdn/ncspot) - Spotify terminal client
- Fairly large projects
  - [hyperswitch](https://github.com/juspay/hyperswitch) - Open Payment Gateway (By Juspay)
  - [rustdesk](https://github.com/rustdesk/rustdesk) - Remote Desktop (Same as Teamviewer)
- Also check
  - https://lib.rs/ - index of crates.io
  - [Awesome-rust](https://github.com/rust-unofficial/awesome-rust)
  - [Idiomatic rust index](https://github.com/mre/idiomatic-rust)
  - Github ([search](https://github.com/search?q=language%3ARust++stars%3A%3C500+&type=repositories) and [trending](https://github.com/trending/rust?since=daily))