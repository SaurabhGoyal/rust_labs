use std::{
    sync::{
        mpsc::{channel, Sender},
        Arc, Mutex,
    },
    thread::{self, JoinHandle},
};

type Job<T> = Box<dyn FnOnce() -> T + Send + 'static>;

pub struct ThreadPool {
    handles: Vec<JoinHandle<()>>,
}

pub struct ThreadPoolJobSender<T> {
    job_sender: Sender<Job<T>>,
}

impl<T> ThreadPoolJobSender<T> {
    pub fn add(&self, job: Job<T>) {
        self.job_sender.send(job).expect("job send error");
    }
}

impl ThreadPool {
    pub fn new<T: 'static>(cap: usize) -> (Self, ThreadPoolJobSender<T>) {
        assert_ne!(cap, 0);
        let (job_sender, job_receiver) = channel::<Job<T>>();
        let receiver = Arc::new(Mutex::new(job_receiver));
        let mut handles = vec![];
        for _i in 0..cap {
            let rc = Arc::clone(&receiver);
            handles.push(thread::spawn(move || {
                while let Ok(job) = rc.lock().unwrap().recv() {
                    job();
                }
            }));
        }
        (ThreadPool { handles }, ThreadPoolJobSender { job_sender })
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        while let Some(handle) = self.handles.pop() {
            handle.join().unwrap();
        }
    }
}
