#![allow(unused)]
use std::{
    cell,
    io::{self, Write as _},
    num::Wrapping,
    sync::{Arc, Mutex},
    thread::{self, JoinHandle},
    time::Duration,
};

const HANDLE_DELAY: u64 = 300;
const RENDER_DELAY: u64 = 300;

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
        let line_sep = format!("\n{}\n", vec!["-"; (4 * dim + 1)].join(""));
        let mut s = String::new();
        let mut complete = true;
        s.push_str(&line_sep);
        for (i, cell) in self.cells.iter().enumerate() {
            if cell.val == 0 {
                complete = false;
            }
            s.push_str(&format!(
                "{}{: ^3}|{}",
                if (i % dim == 0) { "|" } else { "" },
                cell.val,
                if (i % dim == dim - 1) { &line_sep } else { "" }
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
    dbg!(&cell_values);
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
            candidates: (1..=dim).collect(),
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
        if buf.ends_with("\n") {
            buf.pop();
        }
        let board = parse(&buf);
        let board_arc_mutex = Arc::new(Mutex::new(board));
        let mut handles = vec![];
        let board_arc_mutex_clone = Arc::clone(&board_arc_mutex);
        let s_dim = board_arc_mutex_clone.lock().unwrap().s_dim;
        let dim = s_dim * s_dim;
        for index in 0..(dim * dim) {
            let board_arc_mutex_clone = Arc::clone(&board_arc_mutex);
            handles.push(thread::spawn(move || {
                handle_cell(board_arc_mutex_clone, index)
            }));
        }
        let board_arc_mutex_clone = Arc::clone(&board_arc_mutex);
        handles.push(thread::spawn(move || render(board_arc_mutex_clone)));
        for handle in handles {
            handle.join().unwrap();
        }
    }
}

fn handle_cell(mut board_arc_mutex: Arc<Mutex<Board>>, index: usize) {
    let s_dim = board_arc_mutex.lock().unwrap().s_dim;
    let dim = s_dim * s_dim;
    loop {
        if board_arc_mutex.lock().unwrap().cells[index].val > 0 {
            break;
        }
        let candidates = find_candidates(&board_arc_mutex, index);
        thread::sleep(Duration::from_millis(HANDLE_DELAY));
        board_arc_mutex.lock().unwrap().logs.push(format!(
            "{:?}, {:?} -> {:?}",
            index / dim,
            index % dim,
            candidates,
        ));
        if candidates.len() == 1 {
            board_arc_mutex.lock().unwrap().cells[index].val = candidates[0];
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

fn find_candidates(board_arc_mutex: &Arc<Mutex<Board>>, index: usize) -> Vec<usize> {
    let s_dim = board_arc_mutex.lock().unwrap().s_dim;
    let dim = s_dim * s_dim;
    let mut candidates: Vec<usize> = (0..=(s_dim * s_dim)).collect();
    for di in dependency_cells(s_dim, index) {
        // println!(
        //     "{:?},{:?} -> {:?},{:?}",
        //     index / dim,
        //     index % dim,
        //     di / dim,
        //     di % dim
        // );
        let val = board_arc_mutex.lock().unwrap().cells[di].val;
        if val > 0 {
            candidates[val] = 0;
        }
    }
    candidates.into_iter().filter(|x| x > &usize::MIN).collect()
}

fn clear_screen() {
    print!("{}[2J", 27 as char); // ANSI escape code to clear the screen
    print!("{}[1;1H", 27 as char); // ANSI escape code to move the cursor to the top-left corner
    io::stdout().flush().unwrap(); // Flush stdout to ensure screen is cleared immediately
}

fn render(board: Arc<Mutex<Board>>) {
    loop {
        let (s, c) = board.lock().unwrap().pprint();
        println!("{s}");
        if c {
            println!("Done!");
            break;
        }
        thread::sleep(Duration::from_millis(RENDER_DELAY));
        clear_screen();
    }
}
