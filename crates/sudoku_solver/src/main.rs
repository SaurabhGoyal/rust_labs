use std::{
    env,
    io::{self, Write},
};

use sudoku_solver::SudokuSolver;

fn main() {
    let debug = env::args()
        .collect::<Vec<String>>()
        .contains(&"-d".to_string());
    loop {
        println!("Enter a sudoku, It must be of size n x n, one char per cell from top row to bottom row and left to right for each row, use '.' for empty cell.");
        let mut buf = String::new();
        io::stdin().read_line(&mut buf).unwrap();
        if buf.is_empty() {
            break;
        }
        if buf.ends_with('\n') {
            buf.pop();
        }
        let (mut _solver, control, data) = SudokuSolver::new(buf);
        for game_state in data {
            println!("Event - {:?}", game_state.event.message);
            if game_state.event.message.contains("finalisation") {
                let s = game_state.p_repr;
                println!("{s}");
                if game_state.complete {
                    let s = game_state.repr;
                    println!("{s}");
                    break;
                }
                let mut cmd = String::new();
                if debug {
                    println!("// n: Next");
                    io::stdout().flush().unwrap();
                    io::stdin().read_line(&mut cmd).unwrap();
                    cmd.pop();
                } else {
                    cmd = "n".to_string();
                }
                if cmd.eq("n") {
                    control.send(cmd).expect("control send error");
                } else {
                    break;
                }
            } else {
                control.send("n".to_string()).expect("control send error");
            }
            if game_state.complete {
                break;
            }
        }
    }
}
