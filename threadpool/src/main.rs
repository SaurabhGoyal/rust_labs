use threadpool::ThreadPool;

fn main() {
    let (p, job_q, result_q) = ThreadPool::new::<u64>(8);
    for i in 0..200 {
        job_q.add(Box::new(move || i * i));
    }
    drop(job_q);
    for res in result_q {
        println!("{res}");
    }
}
