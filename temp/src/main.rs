use std::{
    thread,
    time::{Duration, Instant},
};
use tokio::{self, runtime};

fn main() {
    let runtime = runtime::Runtime::new().unwrap();
    runtime.block_on(async {
        tokio::spawn(async { test("t1").await });
        tokio::spawn(async { test("t2").await });
        println!("Tasks spawned!");
    });
}

async fn test(msg: &str) {
    let mut i = 0;
    loop {
        println!("{:?} {msg}", Instant::now());
        thread::sleep(Duration::from_millis(500));
        i += 1;
        if i > 10 {
            break;
        }
    }
}
