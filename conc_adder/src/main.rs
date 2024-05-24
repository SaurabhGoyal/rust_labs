use rayon::prelude::*;
use std::{
    sync::{
        mpsc::{self, SyncSender},
        Arc, Mutex,
    },
    thread::{self, JoinHandle},
    time::Instant,
};

type ival = i64;

macro_rules! time_it {
    ($name:expr, $func:expr) => {
        let start = Instant::now();
        let res = $func;
        println!(
            "Call {:?} took {:?} and produced {:?}",
            $name,
            start.elapsed(),
            res,
        );
    };
}

type Job = Box<dyn FnOnce() + Send + 'static>;

#[derive(Debug)]
struct ThreadPool {
    threads: Vec<JoinHandle<()>>,
    sender: Option<SyncSender<Job>>,
}

impl ThreadPool {
    fn new(cap: usize) -> Self {
        assert_ne!(cap, 0);
        let (sender, receiver) = mpsc::sync_channel::<Job>(1000);
        let receiver_mutex = Arc::new(Mutex::new(receiver));
        let mut threads: Vec<JoinHandle<()>> = Vec::with_capacity(cap);
        for id in 0..cap {
            let receiver_clone = Arc::clone(&receiver_mutex);
            threads.push(thread::spawn(move || loop {
                let job = receiver_clone.lock().unwrap().recv();
                if job.is_ok() {
                    job.unwrap()();
                } else {
                    return;
                }
            }));
        }
        ThreadPool {
            threads,
            sender: Some(sender),
        }
    }

    fn add_job<F>(&self, job: F)
    where
        F: FnOnce() + Send + 'static,
    {
        self.sender.as_ref().unwrap().send(Box::new(job)).unwrap();
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        self.sender = None;
        while let Some(thread) = self.threads.pop() {
            thread.join().unwrap();
        }
    }
}

fn adder_thread(counter_mutex_arc: Arc<Mutex<ival>>, num: ival) {
    *counter_mutex_arc.lock().unwrap() += num;
}

fn add_using_threads(nums: &Vec<ival>) -> ival {
    let counter_arc_mutex = Arc::new(Mutex::new(0));
    let pool: ThreadPool = ThreadPool::new(128);
    for num in nums {
        let counter_clone = Arc::clone(&counter_arc_mutex);
        let num_clone = *num;
        pool.add_job(move || adder_thread(counter_clone, num_clone))
    }
    drop(pool);
    return *counter_arc_mutex.lock().unwrap();
}

fn add_using_rayon(nums: &Vec<ival>) -> ival {
    nums.par_iter().sum()
}

fn main() {
    let nums: Vec<ival> = (2..=2000000).into_iter().collect();
    time_it!("Threads", add_using_threads(&nums));
    time_it!("Rayon", add_using_rayon(&nums));
}
