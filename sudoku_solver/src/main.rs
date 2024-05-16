use chrono::Utc;
use core::num;
use std::{
    collections::HashSet,
    io::{self, Write as _},
    num::Wrapping,
    ops::DerefMut,
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};

const HANDLE_DELAY: u64 = 1000;
const RENDER_DELAY: u64 = 2000;

struct Cell {
    val: usize,
    fixed: bool,
    candidates: HashSet<usize>,
}

struct Board {
    s_dim: usize,
    cells: Vec<Cell>,
    logs: Vec<String>,
}

impl Board {
    fn set(&mut self, index: usize, val: usize) {
        self.cells[index].val = val;
        self.cells[index].candidates.clear();
        self.cells[index].candidates.insert(val);
    }

    fn validate(&self) {
        let dim = self.s_dim * self.s_dim;
        for cat in 0..=2 {
            for index in 0..dim {
                let mut nums: HashSet<usize> = HashSet::new();
                for ci in find_candidate_cells(self.s_dim, cat, index) {
                    let val = self.cells[ci].val;
                    if val > 0 && nums.contains(&val) {
                        panic!("Invalid sudoku - {} (in {}) is already present for cat - {}, index - {}", val, ci, cat, index);
                    }
                    nums.insert(val);
                }
            }
        }
    }

    fn pprint(&self) -> (String, bool) {
        let dim = self.s_dim * self.s_dim;
        let line_sep_single = format!("\n{}\n", vec!["-"; 6 * dim + 1].join(""));
        let line_sep_double = format!("\n{}\n", vec!["⹀"; 6 * dim + 1].join(""));
        let mut s = String::new();
        let mut complete = true;
        s.push_str(&line_sep_double);
        for (i, cell) in self.cells.iter().enumerate() {
            if cell.val == 0 {
                complete = false;
            }
            let val_str = if cell.val == 0 {
                format!("     ")
            } else if cell.fixed {
                format!("`{: ^3}`", cell.val)
            } else {
                format!("{: ^5}", cell.val)
            };
            s.push_str(&format!(
                "{}{}{}{}",
                if i % dim == 0 { "‖" } else { "" },
                val_str,
                if i % self.s_dim == self.s_dim - 1 {
                    "‖"
                } else {
                    "|"
                },
                if i % dim == dim - 1 {
                    if i % (self.s_dim * dim) == (self.s_dim * dim) - 1 {
                        &line_sep_double
                    } else {
                        &line_sep_single
                    }
                } else {
                    ""
                },
            ));
        }
        for log in self.logs.iter().rev().take(10) {
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
            fixed: cv > 0,
            candidates: {
                if cv > 0 {
                    (cv..=cv).collect()
                } else {
                    (1..=dim).collect()
                }
            },
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
        for cat in 0..=2 {
            for index in 0..dim {
                let candidates = find_candidate_cells(s_dim, cat, index);
                for num in 1..=dim {
                    let board_arc_mutex_clone = Arc::clone(&board_arc_mutex);
                    let candidates = candidates.clone();
                    handles.push(thread::spawn(move || {
                        handle_number(board_arc_mutex_clone, s_dim, cat, index, num, candidates)
                    }));
                }
            }
        }
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
                    board.cells[index].candidates.remove(&val);
                }
            }
            let candidates: Vec<&usize> = board.cells[index]
                .candidates
                .iter()
                .filter(|x| **x > usize::MIN)
                .collect();
            if candidates.iter().count() == 1 {
                let candidate = candidates.iter().next().unwrap();
                if board.cells[index].val != **candidate {
                    board.logs.push(format!(
                        "[{:#?}] Cell logic - {:?}, {:?} -> {:?}",
                        Utc::now().to_rfc3339(),
                        index / dim,
                        index % dim,
                        candidates,
                    ));
                    board.set(index, **candidate);
                }
                break;
            }
        }
        thread::sleep(Duration::from_millis(HANDLE_DELAY));
    }
}

fn find_candidate_cells(s_dim: usize, category: usize, index: usize) -> HashSet<usize> {
    let dim = s_dim * s_dim;
    let mut candidates: HashSet<usize> = HashSet::new();
    if category == 0 {
        let index = (index / s_dim) * s_dim * dim + ((index % s_dim) * s_dim);
        let ci: usize = index / dim;
        let cj: usize = index % dim;
        let div_i = ci / s_dim;
        let div_j = cj / s_dim;
        for i in (div_i * s_dim)..((div_i + 1) * s_dim) {
            for j in (div_j * s_dim)..((div_j + 1) * s_dim) {
                candidates.insert(i * dim + j);
            }
        }
    } else if category == 1 {
        let start_index = index * dim;
        for i in start_index..(start_index + dim) {
            candidates.insert(i);
        }
    } else if category == 2 {
        let mut index = index;
        loop {
            candidates.insert(index);
            index += dim;
            if index >= dim * dim {
                break;
            }
        }
    }
    candidates
}

fn handle_number(
    board_arc_mutex: Arc<Mutex<Board>>,
    s_dim: usize,
    cat: usize,
    index: usize,
    number: usize,
    mut candidates: HashSet<usize>,
) {
    let dim = s_dim * s_dim;
    loop {
        {
            let mut mutex_guard = match board_arc_mutex.lock() {
                Ok(mg) => mg,
                Err(e) => {
                    println!("Error in getting lock in handle_number - {number} - {e}");
                    return;
                }
            };
            let board = mutex_guard.deref_mut();
            if candidates.iter().count() <= 1 {
                if candidates.iter().count() == 1 {
                    let candidate: &usize = candidates.iter().next().unwrap();
                    if board.cells[*candidate].val != number {
                        board.logs.push(format!(
                            "[{:#?}] Number logic - {:?} -> {:?}, {:?}",
                            Utc::now().to_rfc3339(),
                            number,
                            candidate / dim,
                            candidate % dim,
                        ));
                        board.set(*candidate, number);
                    }
                }
                break;
            }
            candidates.retain(|c| board.cells[*c].candidates.contains(&number));
            let row = candidates
                .iter()
                .map(|c| c / dim)
                .reduce(|acc, e| if acc == e { acc } else { dim + 1 })
                .unwrap_or(dim + 1);
            if row != dim + 1 {
                let cc = find_candidate_cells(s_dim, 1, row);
                for candidate in cc {
                    if !candidates.contains(&candidate)
                        && board.cells[candidate].candidates.contains(&number)
                    {
                        board.cells[candidate].candidates.remove(&number);
                    }
                }
            }
            let col = candidates
                .iter()
                .map(|c| c % dim)
                .reduce(|acc, e| if acc == e { acc } else { dim + 1 })
                .unwrap_or(dim + 1);
            if col != dim + 1 {
                let cc = find_candidate_cells(s_dim, 2, col);
                for candidate in cc {
                    if !candidates.contains(&candidate)
                        && board.cells[candidate].candidates.contains(&number)
                    {
                        board.cells[candidate].candidates.remove(&number);
                    }
                }
            }
        }
        thread::sleep(Duration::from_millis(HANDLE_DELAY));
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
            board.validate();
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
