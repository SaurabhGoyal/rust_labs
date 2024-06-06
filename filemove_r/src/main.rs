use std::{
    env, fs,
    io::{Read, Write},
    path::Path,
    sync::mpsc::{channel, Receiver, Sender},
    thread,
};

fn main() {
    let args = env::args().collect::<Vec<String>>();
    let (mover, event_receiver) = Mover::new();
    thread::spawn(move || {
        let args = args;
        mover.transfer(String::from(&args[1]), String::from(&args[2]));
    });
    while let Ok(event) = event_receiver.recv() {
        println!("{:?}", event.message);
    }
}

struct Event {
    message: String,
}

struct Mover {
    event_sender: Sender<Event>,
}

impl Mover {
    fn new() -> (Self, Receiver<Event>) {
        let (event_sender, event_receiver) = channel::<Event>();
        (Mover { event_sender }, event_receiver)
    }

    fn transfer(&self, source: String, dest_dir: String) {
        self.event_sender
            .send(Event {
                message: format!("Transfer {:?} -> {:?}", source, dest_dir),
            })
            .unwrap();
        let source_path = Path::new(&source)
            .canonicalize()
            .expect("invalid source path");
        let dest_dir_path = Path::new(&dest_dir)
            .canonicalize()
            .expect("invalid dest dir path");
        let dest_path = dest_dir_path.join(source_path.file_name().unwrap());
        if source_path.is_dir() {
            fs::create_dir(dest_path.clone()).expect(
                "Destination either already exists or does not have the given parent path.",
            );
            self.event_sender
                .send(Event {
                    message: format!("Created dir {:?}", dest_path),
                })
                .unwrap();
            for child in fs::read_dir(source_path).unwrap() {
                let child = child.unwrap();
                self.transfer(
                    child.path().as_path().to_str().unwrap().to_string(),
                    dest_path.to_str().unwrap().to_string(),
                )
            }
            return;
        }
        let mut source_file = fs::OpenOptions::new().read(true).open(source_path).unwrap();
        let mut dest_file = fs::OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(dest_path.clone())
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
                message: format!("Created file {:?}", dest_path),
            })
            .unwrap();
    }
}
