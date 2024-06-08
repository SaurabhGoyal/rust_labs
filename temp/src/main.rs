use std::{
    env, fs,
    path::Path,
    sync::{
        mpsc::{channel, Receiver, Sender},
        Arc, Mutex,
    },
    thread::{self, JoinHandle},
    time::Instant,
};

fn time_it<T: Fn()>(name: String, f: T) {
    let start = Instant::now();
    f();
    eprintln!("Call {:?} took {:?}", name, start.elapsed());
}

fn main() {
    let args = env::args().collect::<Vec<String>>();
    let process: ProcessFunc<()> = Arc::new(|p| println!("{p}"));
    for i in 2..=2 {
        let cap = 1 << i;
        time_it(format!("{cap} threads"), || {
            let (_thread_pool, _results, job_sender) = ThreadPool::new(cap);
            let job_sender = Arc::new(job_sender);
            process_file_path(
                String::from(&args[1]),
                Arc::clone(&process),
                Arc::clone(&job_sender),
            )
        });
    }
}

type Job<T> = Box<dyn FnOnce() -> T + Send + 'static>;
type ProcessFunc<T> = Arc<dyn Fn(String) -> T + Send + Sync + 'static>;

fn process_file_path<T>(source: String, f: ProcessFunc<T>, job_sender: Arc<Sender<Job<T>>>) -> T
where
    T: Send + 'static,
{
    let source_path = Path::new(&source)
        .canonicalize()
        .expect("invalid source path");
    let source_path_str = source_path.as_path().to_str().unwrap().to_string();
    let val = f(source_path_str);
    if source_path.is_dir() {
        for child in fs::read_dir(source_path).unwrap() {
            let child = child.unwrap();
            let job_sender_clone = Arc::clone(&job_sender);
            let f_clone = f.clone();
            job_sender
                .send(Box::new(move || {
                    process_file_path(
                        child.path().as_path().to_str().unwrap().to_string(),
                        f_clone,
                        job_sender_clone,
                    )
                }))
                .unwrap();
        }
    }
    val
}

struct ThreadPool {
    handles: Vec<JoinHandle<()>>,
}

impl ThreadPool {
    fn new<T: Send + 'static>(cap: usize) -> (Self, Receiver<T>, Sender<Job<T>>) {
        assert_ne!(cap, 0);
        let (job_sender, job_receiver) = channel::<Job<T>>();
        let (result_sender, result_receiver) = channel::<T>();
        let result_sender = Arc::new(Mutex::new(result_sender));
        let job_receiver = Arc::new(Mutex::new(job_receiver));
        let mut handles = vec![];
        for _i in 0..cap {
            let job_receiver = Arc::clone(&job_receiver);
            let result_sender = Arc::clone(&result_sender);
            handles.push(thread::spawn(move || {
                while let Ok(job) = job_receiver.lock().unwrap().recv() {
                    let _ = result_sender.lock().unwrap().send(job());
                }
            }));
        }
        (ThreadPool { handles }, result_receiver, job_sender)
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        while let Some(handle) = self.handles.pop() {
            handle.join().unwrap();
        }
    }
}
