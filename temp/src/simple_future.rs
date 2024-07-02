use std::time::{Duration, SystemTime};

trait Future {
    fn poll(&mut self) -> Poll;
}

enum Poll {
    Ready,
    Pending,
}

struct Executor {
    futures: Vec<(Box<dyn Future>, bool)>,
}

impl Executor {
    pub fn run(&mut self) {
        loop {
            let mut pending_futures = false;
            for (future, complete) in self.futures.iter_mut().filter(|(_f, c)| !*c) {
                pending_futures = true;
                match future.poll() {
                    Poll::Ready => {
                        *complete = true;
                    }
                    Poll::Pending => {}
                }
            }
            if !pending_futures {
                break;
            }
        }
    }
}

struct Timer {
    dur: Duration,
    start: SystemTime,
    max: Duration,
    run_at: Option<SystemTime>,
}

impl Timer {
    fn new(dur_ms: u64, max_ms: u64) -> Self {
        Self {
            start: SystemTime::now(),
            dur: Duration::from_millis(dur_ms),
            max: Duration::from_millis(max_ms),
            run_at: None,
        }
    }
}

impl Future for Timer {
    fn poll(&mut self) -> Poll {
        let now = SystemTime::now();
        if now.duration_since(self.start).unwrap() >= self.max {
            return Poll::Ready;
        }
        if self.run_at.is_none()
            || now
                .duration_since(self.run_at.as_ref().unwrap().to_owned())
                .unwrap()
                > self.dur
        {
            println!("{:?} Hello for dur:{:?}", now, self.dur);
            self.run_at = Some(SystemTime::now());
        }
        Poll::Pending
    }
}

pub fn run() {
    let a1 = Box::new(Timer::new(1000, 10000));
    let a2 = Box::new(Timer::new(5000, 20000));
    let mut ex = Executor {
        futures: vec![(a1, false), (a2, false)],
    };
    ex.run();
}
