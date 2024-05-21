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
        let (mut solver, control, data) = SudokuSolver::new(buf);
        for event in data {
            println!("Event {:?}", event);
            if event.message.contains("finalisation") {
                println!("{:?}", solver.pprint());
            }
            let mut cmd = String::new();
            println!("// n: Next, b: Break");
            io::stdin().read_line(&mut cmd).unwrap();
            if cmd.eq("n\n") {
                control.send("n".to_string());
                continue;
            } else if cmd.eq("b\n") {
                control.send("b".to_string());
                solver.close();
                continue 'puzzle;
            } else {
                solver.close();
                exit(0);
            }
        }
    }
}
