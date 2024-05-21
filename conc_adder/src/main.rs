use std::{
    ops::DerefMut,
    sync::{Arc, Mutex},
    thread,
};

fn adder_thread(counter_mutex_arc: Arc<Mutex<i32>>, num: i32) {
    let mut mutex_guard = counter_mutex_arc.lock().unwrap();
    let counter = mutex_guard.deref_mut();
    *counter += num;
}

fn add_using_threads(nums: &Vec<i32>) {
    let counter: i32 = 0;
    let counter_arc_mutex = Arc::new(Mutex::new(counter));
    let mut handles = vec![];
    nums.iter().for_each(|num| {
        let counter_arc_mutex_clone = Arc::clone(&counter_arc_mutex);
        let n = *num;
        handles.push(thread::spawn(move || {
            adder_thread(counter_arc_mutex_clone, n)
        }));
    });
    for handle in handles {
        let _ = handle.join();
    }
    println!("Result is {:?}", counter_arc_mutex);
}

fn main() {
    let nums = (2..=200000).into_iter().collect();
    add_using_threads(&nums);
}
