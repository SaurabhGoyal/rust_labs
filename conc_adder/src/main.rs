use futures::executor::block_on;
use rayon::prelude::*;
use std::{
    sync::{
        mpsc::{self, Sender},
        Arc, Mutex,
    },
    thread::{self, JoinHandle},
    time::Instant,
};

type Ival = i64;

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
    sender: Option<Sender<Job>>,
}

impl ThreadPool {
    fn new(cap: usize) -> Self {
        assert_ne!(cap, 0);
        let (sender, receiver) = mpsc::channel::<Job>();
        let receiver_mutex = Arc::new(Mutex::new(receiver));
        let mut threads: Vec<JoinHandle<()>> = Vec::with_capacity(cap);
        for _id in 0..cap {
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

fn adder_thread(counter_mutex_arc: Arc<Mutex<Ival>>, num: Ival) {
    *counter_mutex_arc.lock().unwrap() += num;
}

fn add_using_threads(pool_size: usize, nums: &Vec<Ival>) -> Ival {
    let counter_arc_mutex = Arc::new(Mutex::new(0));
    let pool: ThreadPool = ThreadPool::new(pool_size);
    for num in nums {
        let counter_clone = Arc::clone(&counter_arc_mutex);
        let num_clone = *num;
        pool.add_job(move || adder_thread(counter_clone, num_clone))
    }
    drop(pool);
    return *counter_arc_mutex.lock().unwrap();
}

fn add_using_rayon(nums: &Vec<Ival>) -> Ival {
    nums.par_iter().sum()
}

// Not IO bound, this is just for trying the async semantics.
async fn adder_async(counter_arc_mutex: Arc<Mutex<Ival>>, num: Ival) {
    *counter_arc_mutex.lock().unwrap() += num;
}

async fn adder_async_join(counter_arc_mutex: Arc<Mutex<Ival>>, nums: &Vec<Ival>) {
    for num in nums {
        let counter_clone = Arc::clone(&counter_arc_mutex);
        let num_clone = *num;
        adder_async(counter_clone, num_clone).await;
    }
}

fn add_using_async(nums: &Vec<Ival>) -> Ival {
    let counter_arc_mutex = Arc::new(Mutex::new(0));
    block_on(adder_async_join(Arc::clone(&counter_arc_mutex), nums));
    return *counter_arc_mutex.lock().unwrap();
}

fn main() {
    let nums: Vec<Ival> = (2..=200000000).into_iter().collect();
    time_it!("Rayon", add_using_rayon(&nums));
    // Interestingly the best time is when we are using a single thread in the pool.
    for ps in (0..=4).map(|p| 2_i32.pow(p)) {
        time_it!(
            format!("Threads with pool-size of {}", ps),
            add_using_threads(ps as usize, &nums)
        );
    }
    time_it!("Async", add_using_async(&nums));
}
