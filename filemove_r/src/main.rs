use futures::executor::block_on;
use std::{
    env, fs,
    io::{Read, Write},
    path::Path,
    sync::{
        atomic::AtomicU64,
        mpsc::{channel, Receiver, Sender},
        Arc,
    },
    thread::{self, JoinHandle},
};

fn main() {
    let args = env::args().collect::<Vec<String>>();
    let (mut mover, event_receiver) = Mover::new(
        String::from(&args[1]),
        String::from(&args[2]),
        args.len() > 3 && args[3] == "-a",
    );
    mover.start();
    let event_thread = thread::spawn(move || {
        while let Ok((event, state)) = event_receiver.recv() {
            println!("{:?}\n{:?}\n-------------\n", event, state);
        }
    });
    drop(mover);
    event_thread.join().unwrap();
}

#[derive(Debug, PartialEq, Eq)]
enum EventType {
    DataCopied(u64),
    PathCompleted,
}

#[derive(Debug)]
struct Event {
    source: String,
    event_type: EventType,
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
        println!("Tree - {:?}", tree_root);
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
        let (source, dest_dir, do_async, event_sender, state) = (
            self.source.clone(),
            self.dest_dir.clone(),
            self.do_async,
            self.event_sender.clone(),
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
                block_on(Self::transfer_async(source, dest_dir, internal_event_tx));
            } else {
                Self::transfer(source, dest_dir, internal_event_tx);
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

    fn copy(source_path: &Path, dest_path: &Path, event_sender: Arc<Sender<Event>>) {
        let mut source_file = fs::OpenOptions::new().read(true).open(source_path).unwrap();
        let mut dest_file = fs::OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(dest_path)
            .unwrap();
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
                    source: source_path.to_str().unwrap().to_string(),
                    event_type: EventType::DataCopied(bytes_read as u64),
                })
                .unwrap();

            buf = [0; 1 << 12];
        }
        event_sender
            .send(Event {
                source: source_path.to_str().unwrap().to_string(),
                event_type: EventType::PathCompleted,
            })
            .unwrap();
    }

    async fn transfer_async(source: String, dest_dir: String, event_sender: Arc<Sender<Event>>) {
        let (source_path, dest_path) = Self::init_path(&source, &dest_dir);
        if source_path.is_dir() {
            fs::create_dir(dest_path.clone()).expect(
                "Destination either already exists or does not have the given parent path.",
            );
            for child in fs::read_dir(source_path).unwrap() {
                let child = child.unwrap();
                Box::pin(Self::transfer_async(
                    child.path().as_path().to_str().unwrap().to_string(),
                    dest_path.to_str().unwrap().to_string(),
                    event_sender.clone(),
                ))
                .await;
            }
            return;
        }
        Self::copy(&source_path, &dest_path, event_sender);
    }

    fn transfer(source: String, dest_dir: String, event_sender: Arc<Sender<Event>>) {
        let (source_path, dest_path) = Self::init_path(&source, &dest_dir);
        if source_path.is_dir() {
            fs::create_dir(dest_path.clone()).expect(
                "Destination either already exists or does not have the given parent path.",
            );
            for child in fs::read_dir(source_path).unwrap() {
                let child = child.unwrap();
                Self::transfer(
                    child.path().as_path().to_str().unwrap().to_string(),
                    dest_path.to_str().unwrap().to_string(),
                    event_sender.clone(),
                );
            }
            return;
        }
        Self::copy(&source_path, &dest_path, event_sender);
    }
}

impl Drop for Mover {
    fn drop(&mut self) {
        if let Some(handle) = self.copier_handle.take() {
            handle.join().unwrap();
        }
    }
}
