use std::env;

use walkdir::WalkDir;

fn main() {
    let args = env::args().collect::<Vec<String>>();
    for entry in WalkDir::new(&args[1]).into_iter() {
        let entry = entry.unwrap();
        let md = entry.metadata();
        println!("{:?} -> {:?}", entry, md);
    }
}
