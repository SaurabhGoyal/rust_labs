use std::time::{Duration, SystemTime};

trait Actor {
    fn ready(&self) -> bool;
    fn run(&mut self);
}

struct Executor {
    actors: Vec<Box<dyn Actor>>,
}

impl Executor {
    pub fn run(&mut self) {
        loop {
            for actor in self.actors.iter_mut() {
                if actor.ready() {
                    actor.run();
                }
            }
        }
    }
}

struct Timer {
    dur: Duration,
    run_at: Option<SystemTime>,
}

impl Timer {
    fn new(dur_ms: u64) -> Self {
        Self {
            dur: Duration::from_millis(dur_ms),
            run_at: None,
        }
    }
}

impl Actor for Timer {
    fn ready(&self) -> bool {
        match self.run_at {
            Some(run_at) => SystemTime::now().duration_since(run_at).unwrap() > self.dur,
            None => true,
        }
    }

    fn run(&mut self) {
        println!("Hello for dur:{:?}", self.dur);
        self.run_at = Some(SystemTime::now());
    }
}

pub fn run() {
    let a1 = Box::new(Timer::new(1000));
    let a2 = Box::new(Timer::new(5000));
    let mut ex = Executor {
        actors: vec![a1, a2],
    };
    ex.run();
}
