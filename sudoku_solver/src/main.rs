use std::{fs::read, io, io::Write, process::exit};

use sudoku_solver::SudokuSolver;

fn clear_screen() {
    print!("{}[2J", 27 as char); // ANSI escape code to clear the screen
    print!("{}[1;1H", 27 as char); // ANSI escape code to move the cursor to the top-left corner
    io::stdout().flush().unwrap(); // Flush stdout to ensure screen is cleared immediately
}

fn main() {
    'puzzle: loop {
        println!("Enter a sudoku, It must be of size nxn and in a comma separated value set for all cells in a single line");
        let mut buf = String::new();
        io::stdin().read_line(&mut buf).unwrap();
        if buf.is_empty() {
            break;
        }
        if buf.ends_with("\n") {
            buf.pop();
        }
        let (mut _solver, control, data) = SudokuSolver::new(buf);
        for game_state in data {
            println!("Event - {:?}", game_state.event.message);
            if game_state.event.message.contains("finalisation") {
                let s = game_state.p_repr;
                println!("{s}");
                if game_state.complete {
                    break;
                }
                // println!("// n: Next, b: Break");
                // let mut cmd = String::new();
                // io::stdout().flush().unwrap();
                // io::stdin().read_line(&mut cmd).unwrap();
                // cmd.pop();
                let cmd = "n".to_string();
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
