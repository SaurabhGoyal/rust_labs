mod actor;
mod simple_future;
mod tcp_concurrency;
mod tokio_single_thread;

fn main() {
    // tcp_concurrency::run();
    // actor::run();
    // simple_future::run();
    tokio_single_thread::run(); // DOES NOT WORK - ONLY one task is executed, no concurrency.
}
