#![feature(test)]
extern crate test;

mod sudoku;

// pub use sudoku::parse;
pub use sudoku::SudokuSolver;

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use test::Bencher;

    #[test]
    fn correctness() {
        for (i, line) in fs::read_to_string(
            // "/mnt/d/Saurabh/Personal/rust_labs/sudoku_solver/src/test_data/local/medium.txt",
            "/mnt/d/Saurabh/Personal/rust_labs/sudoku_solver/src/test_data/sudoku_set.csv",
        )
        .unwrap()
        .split("\n")
        .enumerate()
        {
            let parts = line.split(",").collect::<Vec<&str>>();
            let puzzle = parts[1];
            let soln = parts[2];
            let solver = SudokuSolver::new(puzzle.to_string());
            let solution = solve(puzzle.to_string());
            assert_eq!(
                soln,
                solution
                    .unwrap()
                    .cells
                    .iter()
                    .map(|v| v.to_string())
                    .collect::<Vec<String>>()
                    .join("")
            );
            if i >= 1 {
                break;
            }
        }
    }

    #[bench]
    fn benchmark(b: &mut Bencher) {
        for line in fs::read_to_string(
            "/mnt/d/Saurabh/Personal/rust_labs/sudoku_solver/src/test_data/local/medium.txt",
        )
        .unwrap()
        .split("\n")
        {
            let parts = line.split(",").collect::<Vec<&str>>();
            let puzzle = parts[1];
            println!("Checking {puzzle}");
            b.iter(|| {
                solve(puzzle.to_string());
            });
        }
    }
}
