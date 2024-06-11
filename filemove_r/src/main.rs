use futures::executor::block_on;
use std::{
    env, fmt, fs,
    io::{self, Read, Write},
    path::{Display, Path},
    sync::{
        atomic::AtomicU64,
        mpsc::{channel, Receiver, Sender},
        Arc,
    },
    thread::{self, JoinHandle},
    time::{Duration, Instant},
};

fn clear_screen() {
    print!("{}[2J", 27 as char); // ANSI escape code to clear the screen
    print!("{}[1;1H", 27 as char); // ANSI escape code to move the cursor to the top-left corner
    io::stdout().flush().unwrap(); // Flush stdout to ensure screen is cleared immediately
}

fn main() {
    let args = env::args().collect::<Vec<String>>();
    let (mut mover, event_receiver) = Mover::new(
        String::from(&args[1]),
        String::from(&args[2]),
        args.len() > 3 && args[3] == "-a",
    );
    mover.start();
    let render_thread = thread::spawn(move || {
        let mut last_ts = Instant::now();
        while let Ok((event, state)) = event_receiver.recv() {
            let now = Instant::now();
            if now.duration_since(last_ts).as_millis() >= 200
                || state
                    .copied_bytes
                    .load(std::sync::atomic::Ordering::Relaxed)
                    == state.total_bytes
            {
                clear_screen();
                println!("{state}-------------\n{event}\n");
                last_ts = now;
            }
        }
    });
    drop(mover);
    render_thread.join().unwrap();
}

#[derive(Debug, PartialEq, Eq)]
enum EventType {
    DataCopied(u64),
    PathCompleted,
}

#[derive(Debug)]
struct Event {
    path: String,
    event_type: EventType,
}

impl fmt::Display for Event {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut out = String::new();
        if self.event_type == EventType::PathCompleted {
            out.push_str(format!("File `{}` copied successfully.", self.path).as_str());
        } else {
            out.push_str(format!("File `{}` is being copied.", self.path).as_str());
        }
        f.write_str(out.as_str())
    }
}

#[derive(Debug)]
struct State {
    total_count: u64,
    file_count: u64,
    folder_count: u64,
    total_bytes: u64,
    copied_file_count: AtomicU64,
    copied_bytes: AtomicU64,
}

impl fmt::Display for State {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut out = String::new();
        let total = self.total_bytes;
        let copied = self.copied_bytes.load(std::sync::atomic::Ordering::Relaxed);
        let progress = (copied as f64 / total as f64) * 100.0;
        let progress_char_size = progress as usize;
        out.push_str(
            format!(
                "[{}>{}]\n",
                vec!["="; progress_char_size].join(""),
                vec![" "; 100 - progress_char_size].join(""),
            )
            .as_str(),
        );
        out.push_str(
            format!(
                "- Total files - {}\n- Copied - {}\n- Progress - {:.2}%\n",
                total, copied, progress
            )
            .as_str(),
        );
        f.write_str(out.as_str())
    }
}

struct Mover {
    source: String,
    dest_dir: String,
    do_async: bool,
    tree_root: Arc<win_tree::TreeNode>,
    state: Arc<State>,
    copier_handle: Option<JoinHandle<()>>,
    event_handle: Option<JoinHandle<()>>,
    event_sender: Arc<Sender<(Event, Arc<State>)>>,
}

impl Mover {
    fn new(
        source: String,
        dest_dir: String,
        do_async: bool,
    ) -> (Self, Receiver<(Event, Arc<State>)>) {
        let (_source_path, _dest_path) = Self::init_path(&source, &dest_dir);
        let tree_root = Arc::new(
            win_tree::build(win_tree::Config {
                path: source.to_string(),
                depth_check: None,
                exclude_pattern: None,
                build_method: win_tree::BuildMethod::ParallelRayon,
            })
            .expect("unable to build tree"),
        );
        let (event_sender, event_receiver) = channel::<(Event, Arc<State>)>();
        let event_sender = Arc::new(event_sender);
        fn get_counts(node: Arc<win_tree::TreeNode>) -> (u64, u64, u64) {
            if node.is_file {
                return (1, 1, 0);
            }
            let (mut total, mut files, mut folders) = (1, 0, 1);
            for c in &node.children {
                let (c_total, c_files, c_folders) = get_counts(c.clone());
                total += c_total;
                files += c_files;
                folders += c_folders;
            }
            (total, files, folders)
        }
        let (total, files, folders) = get_counts(tree_root.clone());
        let state = State {
            total_count: total,
            file_count: files,
            folder_count: folders,
            total_bytes: tree_root.size_in_bytes.clone().take().unwrap_or(0),
            copied_file_count: 0.into(),
            copied_bytes: 0.into(),
        };
        (
            Mover {
                source,
                dest_dir,
                do_async,
                event_sender,
                tree_root,
                state: Arc::new(state),
                copier_handle: None,
                event_handle: None,
            },
            event_receiver,
        )
    }

    fn start(&mut self) {
        let (source, dest_dir, do_async, event_sender, tree_root, state) = (
            self.source.clone(),
            self.dest_dir.clone(),
            self.do_async,
            self.event_sender.clone(),
            self.tree_root.clone(),
            self.state.clone(),
        );
        let (internal_event_tx, internal_event_rx) = channel::<Event>();
        let internal_event_tx = Arc::new(internal_event_tx);
        self.event_handle = Some(thread::spawn(move || {
            let state = state;
            while let Ok(event) = internal_event_rx.recv() {
                match event.event_type {
                    EventType::DataCopied(bytes_copied) => {
                        state
                            .copied_bytes
                            .fetch_add(bytes_copied, std::sync::atomic::Ordering::Relaxed);
                    }
                    EventType::PathCompleted => {
                        state
                            .copied_file_count
                            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                    }
                }
                event_sender.send((event, state.clone())).unwrap();
            }
        }));
        self.copier_handle = Some(thread::spawn(move || {
            if do_async {
                block_on(Self::transfer_async(
                    source,
                    dest_dir,
                    tree_root.clone(),
                    internal_event_tx,
                ));
            } else {
                Self::transfer(source, dest_dir, tree_root.clone(), internal_event_tx);
            }
        }));
    }

    fn init_path(source: &String, dest_dir: &String) -> (std::path::PathBuf, std::path::PathBuf) {
        let source_path = Path::new(&source)
            .canonicalize()
            .expect("invalid source path");
        let dest_path = Path::new(&dest_dir)
            .canonicalize()
            .expect("invalid dest dir path")
            .join(source_path.file_name().unwrap());
        (source_path, dest_path)
    }

    fn copy(source_path: &String, dest_path: &String, event_sender: Arc<Sender<Event>>) {
        let mut source_file = fs::OpenOptions::new().read(true).open(source_path).unwrap();
        let mut dest_file = fs::OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(dest_path)
            .expect(format!("issue in dest path {dest_path}").as_str());
        // copy(&mut source_file, &mut dest_file).expect("error in copying");
        let mut buf: [u8; 4096] = [0; 1 << 12];
        loop {
            let bytes_read = source_file.read(&mut buf).expect("error in reading file");
            if bytes_read == 0 {
                break;
            }
            dest_file
                .write_all(&buf[0..bytes_read])
                .expect("error in writing to file");
            event_sender
                .send(Event {
                    path: String::from(source_path),
                    event_type: EventType::DataCopied(bytes_read as u64),
                })
                .unwrap();

            buf = [0; 1 << 12];
        }
        event_sender
            .send(Event {
                path: String::from(source_path),
                event_type: EventType::PathCompleted,
            })
            .unwrap();
    }

    async fn transfer_async(
        source: String,
        dest_dir: String,
        tree_node: Arc<win_tree::TreeNode>,
        event_sender: Arc<Sender<Event>>,
    ) {
        if !tree_node.is_file {
            let dest_path = format!("{}/{}", dest_dir, tree_node.name);
            fs::create_dir(&dest_path).expect(
                "Destination either already exists or does not have the given parent path.",
            );
            for child in tree_node.children.iter() {
                Box::pin(Self::transfer_async(
                    format!("{}/{}", source, child.name),
                    dest_path.to_string(),
                    child.clone(),
                    event_sender.clone(),
                ))
                .await;
            }
            return;
        }
        Self::copy(
            &source,
            &format!("{}/{}", dest_dir, tree_node.name),
            event_sender,
        );
    }

    fn transfer(
        source: String,
        dest_dir: String,
        tree_node: Arc<win_tree::TreeNode>,
        event_sender: Arc<Sender<Event>>,
    ) {
        if !tree_node.is_file {
            let dest_path = format!("{}/{}", dest_dir, tree_node.name);
            fs::create_dir(&dest_path).expect(
                "Destination either already exists or does not have the given parent path.",
            );
            for child in tree_node.children.iter() {
                Self::transfer(
                    format!("{}/{}", source, child.name),
                    dest_path.to_string(),
                    child.clone(),
                    event_sender.clone(),
                );
            }
            return;
        }
        Self::copy(
            &source,
            &format!("{}/{}", dest_dir, tree_node.name),
            event_sender,
        );
    }
}

impl Drop for Mover {
    fn drop(&mut self) {
        if let Some(handle) = self.copier_handle.take() {
            handle.join().unwrap();
        }
    }
}
