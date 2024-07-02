use std::time::{Duration, SystemTime};

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let handle_1 = tokio::spawn(async move {
        loop {
            println!("{:?} Hello from 1", SystemTime::now());
            tokio::time::sleep(Duration::from_millis(2000)).await;
        }
    });
    let handle_2 = tokio::spawn(async move {
        loop {
            println!("{:?} Hello from 2", SystemTime::now());
            tokio::time::sleep(Duration::from_millis(5000)).await;
        }
    });
    handle_2.await.unwrap();
    handle_1.await.unwrap();
}

pub fn run() {
    main()
}
