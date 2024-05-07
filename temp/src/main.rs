use std::thread;

fn main() {
    let v = vec![1, 4, 5];
    let c = move || println!("{:?}", v);
    thread::spawn(c).join().unwrap();
}
