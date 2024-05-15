use chrono::Utc;
use std::{
    io::{self, Write as _},
    num::Wrapping,
    ops::DerefMut,
    sync::{Arc, Mutex},
    thread::{self},
    time::Duration,
};

const HANDLE_DELAY: u64 = 1000;
const RENDER_DELAY: u64 = 500;

struct Cell {
    val: usize,
    candidates: Vec<usize>,
}

struct Board {
    s_dim: usize,
    cells: Vec<Cell>,
    logs: Vec<String>,
}

impl Board {
    fn pprint(&self) -> (String, bool) {
        let dim = self.s_dim * self.s_dim;
        let line_sep = format!("\n{}\n", vec!["-"; 4 * dim + 1].join(""));
        let mut s = String::new();
        let mut complete = true;
        s.push_str(&line_sep);
        for (i, cell) in self.cells.iter().enumerate() {
            if cell.val == 0 {
                complete = false;
            }
            s.push_str(&format!(
                "{}{: ^3}|{}",
                if i % dim == 0 { "|" } else { "" },
                cell.val,
                if i % dim == dim - 1 { &line_sep } else { "" }
            ));
        }
        for log in &self.logs {
            s.push_str(&format!("{}\n", log));
        }
        (s, complete)
    }
}

fn is_perfect_square(n: usize) -> bool {
    let sqrt_n = (n as f64).sqrt() as usize;
    let squared = Wrapping(sqrt_n) * Wrapping(sqrt_n);
    squared.0 == n
}

fn parse(buf: &str) -> Board {
    let parse_cell = |cv: &str| -> usize {
        if cv.len() == 0 {
            return 0;
        }
        return cv.parse::<usize>().unwrap();
    };
    let cell_values: Vec<usize> = buf.split(",").map(parse_cell).collect();
    let cell_count = cell_values.len();
    let dim = (cell_count as f64).sqrt() as usize;
    let s_dim = (dim as f64).sqrt() as usize;
    if !is_perfect_square(cell_count) || !is_perfect_square(dim) {
        println!("{cell_count}, {dim}, {s_dim}");
        panic!("Invalid dimension");
    }
    let cells = cell_values
        .into_iter()
        .map(|cv| Cell {
            val: cv,
            candidates: (0..=dim).collect(),
        })
        .collect();
    Board {
        s_dim,
        cells,
        logs: vec![],
    }
}

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
        let board = parse(&buf);
        let s_dim = board.s_dim;
        let dim = s_dim * s_dim;
        let board_arc_mutex = Arc::new(Mutex::new(board));
        let mut handles = vec![];
        for index in 0..(dim * dim) {
            let board_arc_mutex_clone = Arc::clone(&board_arc_mutex);
            handles.push(thread::spawn(move || {
                handle_cell(board_arc_mutex_clone, s_dim, index)
            }));
        }
        // for index in 0..dim {
        //     for num in 1..=dim {
        //         let board_arc_mutex_clone = Arc::clone(&board_arc_mutex);
        //         handles.push(thread::spawn(move || {
        //             handle_number(board_arc_mutex_clone, index, num)
        //         }));
        //     }
        // }
        let board_arc_mutex_clone = Arc::clone(&board_arc_mutex);
        handles.push(thread::spawn(move || render(board_arc_mutex_clone)));
        for handle in handles {
            let _ = handle.join();
        }
    }
}

fn handle_cell(board_arc_mutex: Arc<Mutex<Board>>, s_dim: usize, index: usize) {
    let dim = s_dim * s_dim;
    let dcells = dependency_cells(s_dim, index);
    loop {
        {
            let mut mutex_guard = match board_arc_mutex.lock() {
                Ok(mg) => mg,
                Err(e) => {
                    println!("Error in getting lock in handle_cell - {index} - {e}");
                    return;
                }
            };
            let board = mutex_guard.deref_mut();
            for di in &dcells {
                let val = board.cells[*di].val;
                if val > 0 {
                    board.cells[index].candidates[val] = 0;
                }
            }
            let candidates: Vec<&usize> = board.cells[index]
                .candidates
                .iter()
                .filter(|x| **x > usize::MIN)
                .collect();
            if candidates.len() == 1 {
                board.logs.push(format!(
                    "[{:#?}] {:?}, {:?} -> {:?}",
                    Utc::now().to_rfc3339(),
                    index / dim,
                    index % dim,
                    candidates,
                ));
                board.cells[index].val = *candidates[0];
                break;
            }
        }
        thread::sleep(Duration::from_millis(HANDLE_DELAY));
    }
}

fn handle_number(board_arc_mutex: Arc<Mutex<Board>>, box_index: usize, number: usize) {
    let s_dim = board_arc_mutex.lock().unwrap().s_dim;
    let dim = s_dim * s_dim;
    let index = (box_index / s_dim) * s_dim * dim + ((box_index % s_dim) * s_dim);
    let ci: usize = index / dim;
    let cj: usize = index % dim;
    let div_i = ci / s_dim;
    let div_j = cj / s_dim;
    let mut candidates: Vec<usize> = vec![];
    for i in (div_i * s_dim)..((div_i + 1) * s_dim) {
        for j in (div_j * s_dim)..((div_j + 1) * s_dim) {
            candidates.push(i * dim + j);
        }
    }
    loop {
        if candidates.len() == 1 {
            board_arc_mutex.lock().unwrap().cells[candidates[0]].val = number;
            break;
        }
        for i in (div_i * s_dim)..((div_i + 1) * s_dim) {
            for j in (div_j * s_dim)..((div_j + 1) * s_dim) {
                if !board_arc_mutex.lock().unwrap().cells[i * dim + j]
                    .candidates
                    .contains(&number)
                {
                    candidates.remove(i * dim + j);
                }
                if number == 7 && box_index == 0 {
                    println!(
                        "Cell {:?},{:?} has candidates {:?}",
                        i,
                        j,
                        board_arc_mutex.lock().unwrap().cells[i * dim + j].candidates,
                    );
                }
            }
        }
        if number == 7 && box_index == 0 {
            println!(
                "Candidates for {:?} in Box-{:?} -> {:?}",
                number,
                box_index + 1,
                candidates
            );
        }
    }
}

fn dependency_cells(s_dim: usize, index: usize) -> Vec<usize> {
    let mut dcells: Vec<usize> = vec![];
    let dim = s_dim * s_dim;
    let ci: usize = index / dim;
    let cj: usize = index % dim;
    for i in 0..dim {
        if i != ci {
            dcells.push(i * dim + cj);
        }
    }
    for j in 0..dim {
        if j != cj {
            dcells.push(ci * dim + j);
        }
    }
    let div_i = ci / s_dim;
    let div_j = cj / s_dim;
    for i in (div_i * s_dim)..((div_i + 1) * s_dim) {
        for j in (div_j * s_dim)..((div_j + 1) * s_dim) {
            if !(i == ci && j == cj) {
                dcells.push(i * dim + j);
            }
        }
    }
    dcells
}

fn clear_screen() {
    print!("{}[2J", 27 as char); // ANSI escape code to clear the screen
    print!("{}[1;1H", 27 as char); // ANSI escape code to move the cursor to the top-left corner
    io::stdout().flush().unwrap(); // Flush stdout to ensure screen is cleared immediately
}

fn render(board_arc_mutex: Arc<Mutex<Board>>) {
    loop {
        {
            let mut mutex_guard = match board_arc_mutex.lock() {
                Ok(mg) => mg,
                Err(e) => {
                    println!("Error in getting lock in render - {e}");
                    return;
                }
            };
            let board = mutex_guard.deref_mut();
            let (s, c) = board.pprint();
            println!("{s}");
            if c {
                println!("Done!");
                break;
            }
        }
        thread::sleep(Duration::from_millis(RENDER_DELAY));
        clear_screen();
    }
}
