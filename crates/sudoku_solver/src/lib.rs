mod sudoku;

// pub use sudoku::parse;
pub use sudoku::SudokuSolver;

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn correctness() {
        for (i, line) in fs::read_to_string(
            "/mnt/d/Saurabh/Personal/rust_labs/sudoku_solver/src/test_data/medium.txt",
            // "/mnt/d/Saurabh/Personal/rust_labs/sudoku_solver/src/test_data/sudoku_set.csv",
        )
        .unwrap()
        .split('\n')
        .enumerate()
        {
            let parts = line.split(",").collect::<Vec<&str>>();
            let puzzle = parts[1];
            let soln = parts[2];
            let (mut _solver, control, data) = SudokuSolver::new(puzzle.to_string());
            let mut solution = String::new();
            for game_state in data {
                if game_state.complete {
                    solution = game_state.repr;
                    break;
                }
                control.send("n".to_string()).unwrap();
            }
            assert_eq!(soln, solution);
            if i >= 10 {
                break;
            }
        }
    }
}
