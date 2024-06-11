use std::{
    sync::{
        mpsc::{channel, Receiver, Sender},
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
    pub fn new<T: Send + 'static>(cap: usize) -> (Self, ThreadPoolJobSender<T>, Receiver<T>) {
        assert_ne!(cap, 0);
        let (job_sender, job_receiver) = channel::<Job<T>>();
        let (result_sender, result_receiver) = channel::<T>();
        let job_receiver = Arc::new(Mutex::new(job_receiver));
        let result_sender = Arc::new(Mutex::new(result_sender));
        let mut handles = vec![];
        for _i in 0..cap {
            let job_receiver = Arc::clone(&job_receiver);
            let result_sender = Arc::clone(&result_sender);
            handles.push(thread::spawn(move || {
                while let Ok(job) = job_receiver.lock().unwrap().recv() {
                    result_sender.lock().unwrap().send(job()).unwrap();
                }
            }));
        }
        (
            ThreadPool { handles },
            ThreadPoolJobSender { job_sender },
            result_receiver,
        )
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        while let Some(handle) = self.handles.pop() {
            handle.join().unwrap();
        }
    }
}
