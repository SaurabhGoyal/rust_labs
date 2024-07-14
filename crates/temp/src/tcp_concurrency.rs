use std::{
    env,
    io::{Read, Write},
    net::TcpStream,
    thread,
    time::Duration,
};

pub fn run() {
    let args = env::args().collect::<Vec<String>>();
    let stream = TcpStream::connect((args[1].as_str(), args[2].parse::<u16>().unwrap())).unwrap();
    let mut stream_tx = stream.try_clone().unwrap();
    let mut stream_rx = stream.try_clone().unwrap();
    let handle_tx = thread::spawn(move || {
        let mut i = 0;
        loop {
            let _wb = stream_tx
                .write(format!("hello - {i}\n").as_bytes())
                .unwrap();
            i += 1;
            thread::sleep(Duration::from_secs(10));
        }
    });
    let handle_rx = thread::spawn(move || loop {
        let mut buf = [0_u8; 100];
        let rb = stream_rx.read(&mut buf).unwrap();
        println!(
            "Read {rb} bytes - {}",
            std::str::from_utf8(&buf[..rb]).unwrap()
        );
    });
    handle_tx.join().unwrap();
    handle_rx.join().unwrap();
}
