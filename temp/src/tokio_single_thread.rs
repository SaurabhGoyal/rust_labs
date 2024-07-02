use std::{
    thread,
    time::{Duration, SystemTime},
};

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let handle_1 = tokio::spawn(async move {
        loop {
            println!("{:?} Hello from 1", SystemTime::now());
            thread::sleep(Duration::from_millis(2000))
        }
    });
    let handle_2 = tokio::spawn(async move {
        loop {
            println!("{:?} Hello from 2", SystemTime::now());
            thread::sleep(Duration::from_millis(5000))
        }
    });
    handle_2.await.unwrap();
    handle_1.await.unwrap();
}

pub fn run() {
    main()
}
