use std::io;

use sudoku_solver::solve;

fn main() {
    loop {
        println!("Enter a sudoku, It must be of size nxn and in a comma separated value set for all cells in a single line");
        let mut buf = String::new();
        io::stdin().read_line(&mut buf).unwrap();
        if buf.is_empty() {
            break;
        }
        if buf.ends_with("\n") {
            buf.pop();
        }
        let _ = solve(buf);
        // println!("{:?}", solution);
    }
}
