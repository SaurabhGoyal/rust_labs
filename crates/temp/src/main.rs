mod actor;
mod hashing;
mod itertools;
mod simple_future;
mod tcp_concurrency;
mod tokio_single_thread;

fn main() {
    // tcp_concurrency::run();
    // actor::run();
    // simple_future::run();
    // tokio_single_thread::run();
    // hashing::run();
    itertools::run();
}
