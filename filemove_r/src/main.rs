use futures::executor::block_on;
use std::{
    env, fs,
    io::{Read, Write},
    path::Path,
    sync::{atomic::AtomicUsize, mpsc::{channel, Receiver, Sender}},
    thread,
};

fn main() {
    let args = env::args().collect::<Vec<String>>();
    let (mover, event_receiver) = Mover::new();
    let transfer_thread = thread::spawn(move || {
        let args = args;
        if args.len() > 3 && args[3] == "-a" {
            println!("<------------- Async ------------->");
            block_on(mover.transfer_async(String::from(&args[1]), String::from(&args[2])));
        } else {
            mover.transfer(String::from(&args[1]), String::from(&args[2]));
        }
    });
    let event_thread = thread::spawn(move || {
        while let Ok(event) = event_receiver.recv() {
            if event.event_type == EventType::CompletedCopying {
                println!("{:?}", event);
            }
        }
    });
    transfer_thread.join().unwrap();
    event_thread.join().unwrap();
}

#[derive(Debug, PartialEq, Eq)]
enum EventType {
    StartedCopying,
    CompletedCopying,
}

#[derive(Debug)]
struct Event {
    source: String,
    event_type: EventType,
}

struct MoverState {
    file_count: usize,
    folder_count: usize,
    total_bytes: usize,
    copied_file_count: AtomicUsize,
    copied_bytes: AtomicUsize,
}

struct Mover {
    state: {
    },
    event_sender: Sender<Event>,
}

impl Mover {
    fn new() -> (Self, Receiver<Event>) {
        let (event_sender, event_receiver) = channel::<Event>();
        (Mover { event_sender }, event_receiver)
    }

    fn init_path(
        &self,
        source: &String,
        dest_dir: &String,
    ) -> (std::path::PathBuf, std::path::PathBuf) {
        self.event_sender
            .send(Event {
                source: source.to_string(),
                event_type: EventType::StartedCopying,
            })
            .unwrap();
        let source_path = Path::new(&source)
            .canonicalize()
            .expect("invalid source path");
        let dest_path = Path::new(&dest_dir)
            .canonicalize()
            .expect("invalid dest dir path")
            .join(source_path.file_name().unwrap());
        (source_path, dest_path)
    }

    fn copy(&self, source_path: &Path, dest_path: &Path) {
        let mut source_file = fs::OpenOptions::new().read(true).open(source_path).unwrap();
        let mut dest_file = fs::OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(dest_path)
            .unwrap();
        // copy(&mut source_file, &mut dest_file).expect("error in copying");
        let mut buf = [0; 1 << 12];
        loop {
            let bytes_read = source_file.read(&mut buf).expect("error in reading file");
            if bytes_read == 0 {
                break;
            }
            dest_file
                .write_all(&buf[0..bytes_read])
                .expect("error in writing to file");
            buf = [0; 1 << 12];
        }
        self.event_sender
            .send(Event {
                source: source_path.to_str().unwrap().to_string(),
                event_type: EventType::CompletedCopying,
            })
            .unwrap();
    }

    async fn transfer_async(&self, source: String, dest_dir: String) {
        let (source_path, dest_path) = self.init_path(&source, &dest_dir);
        if source_path.is_dir() {
            fs::create_dir(dest_path.clone()).expect(
                "Destination either already exists or does not have the given parent path.",
            );
            for child in fs::read_dir(source_path).unwrap() {
                let child = child.unwrap();
                Box::pin(self.transfer_async(
                    child.path().as_path().to_str().unwrap().to_string(),
                    dest_path.to_str().unwrap().to_string(),
                ))
                .await;
            }
            return;
        }
        self.copy(&source_path, &dest_path);
    }

    fn transfer(&self, source: String, dest_dir: String) {
        let (source_path, dest_path) = self.init_path(&source, &dest_dir);
        if source_path.is_dir() {
            fs::create_dir(dest_path.clone()).expect(
                "Destination either already exists or does not have the given parent path.",
            );
            for child in fs::read_dir(source_path).unwrap() {
                let child = child.unwrap();
                self.transfer(
                    child.path().as_path().to_str().unwrap().to_string(),
                    dest_path.to_str().unwrap().to_string(),
                );
            }
            return;
        }
        self.copy(&source_path, &dest_path);
    }
}
