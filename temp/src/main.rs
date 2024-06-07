use futures::executor::block_on;
use std::{env, fs, path::Path, rc::Rc};

fn main() {
    let args = env::args().collect::<Vec<String>>();
    let process: ProcessFunc = Rc::new(|p| println!("{p}"));
    block_on(process_file_path(
        String::from(&args[1]),
        Rc::clone(&process),
    ));
}

type ProcessFunc = Rc<dyn Fn(String) + Send + 'static>;

async fn process_file_path(source: String, f: ProcessFunc) {
    let source_path = Path::new(&source)
        .canonicalize()
        .expect("invalid source path");
    let source_path_str = source_path.as_path().to_str().unwrap().to_string();
    f(source_path_str);
    if source_path.is_dir() {
        for child in fs::read_dir(source_path).unwrap() {
            let child = child.unwrap();
            Box::pin(process_file_path(
                child.path().as_path().to_str().unwrap().to_string(),
                Rc::clone(&f),
            ))
            .await;
        }
    }
}
