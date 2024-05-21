use std::{
    collections::{HashMap, HashSet},
    io::{self, Write as _},
    num::Wrapping,
    ops::{Deref, DerefMut},
    sync::{
        mpsc::{sync_channel, Iter, Receiver, SyncSender},
        Arc, Mutex,
    },
    thread::{self, JoinHandle},
    time::Duration,
};

use chrono::Utc;

const HANDLE_DELAY: u64 = 2500;

struct Cell {
    val: usize,
    fixed: bool,
    candidates: HashSet<usize>,
}

#[derive(Debug)]
pub struct Event {
    pub message: String,
}

pub struct SudokuSolver {
    buf: String,
    board_arc_mutex: Arc<Mutex<Board>>,
    thread_handles: Vec<JoinHandle<()>>,
}

impl SudokuSolver {
    pub fn new(mut buf: String) -> (Self, SyncSender<String>, Receiver<Event>) {
        // validate
        if buf.is_empty() {
            panic!("Empty string");
        }
        if buf.ends_with("\n") {
            buf.pop();
        }
        // initialise channels and board
        let (data_sender, data_receiver) = sync_channel::<Event>(1);
        let (control_sender, control_receiver) = sync_channel::<String>(1);
        let board = parse(&buf, data_sender, control_receiver);

        // initialise threads
        let mut handles = vec![];
        let board_arc_mutex = Arc::new(Mutex::new(board));
        let s_dim = Arc::clone(&board_arc_mutex).lock().unwrap().s_dim;
        let dim = s_dim * s_dim;
        for index in 0..(dim * dim) {
            let board_arc_mutex_clone = Arc::clone(&board_arc_mutex);
            handles.push(thread::spawn(move || {
                handle_cell(board_arc_mutex_clone, s_dim, index)
            }));
        }
        for num in 1..=dim {
            for cat in 0..=2 {
                for index in 0..dim {
                    let board_arc_mutex_clone = Arc::clone(&board_arc_mutex);
                    handles.push(thread::spawn(move || {
                        handle_number(board_arc_mutex_clone, s_dim, num, cat, index)
                    }));
                }
            }
        }
        (
            SudokuSolver {
                buf,
                board_arc_mutex,
                thread_handles: handles,
            },
            control_sender,
            data_receiver,
        )
    }

    pub fn pprint(&self) -> (String, bool) {
        Arc::clone(&self.board_arc_mutex)
            .lock()
            .unwrap()
            .deref()
            .pprint()
    }

    pub fn close(&mut self) {
        loop {
            let handle = self.thread_handles.pop();
            if handle.is_none() {
                break;
            }
            handle.unwrap().join().unwrap();
        }
    }
}

struct Board {
    s_dim: usize,
    cells: Vec<Cell>,
    data_sender: SyncSender<Event>,
    control_receiver: Receiver<String>,
    num_candidates: Vec<Vec<Vec<HashSet<usize>>>>,
    state: u8,
    logs: Vec<String>,
    candidate_mask: HashMap<String, HashSet<usize>>,
}

impl Board {
    fn set(&mut self, actor: usize, index: usize, val: usize, reason: &str) {
        if self.cells[index].val == val {
            return;
        }
        self.validate();
        let dim = self.s_dim * self.s_dim;
        self.cells[index].val = val;
        self.cells[index].candidates.clear();
        self.cells[index].candidates.insert(val);
        self.post_update(Event {
            message: format!(
                "[{:#?}] Cell candidate finalisation - ({:?}, {:?}) -> {:?} - [Reason -> {:?}]",
                Utc::now().to_rfc3339(),
                index / dim,
                index % dim,
                val,
                reason,
            ),
        });
    }

    fn remove_cell_candidate(&mut self, actor: usize, index: usize, val: usize, reason: &str) {
        let dim = self.s_dim * self.s_dim;
        self.cells[index].candidates.remove(&val);
        self.post_update(Event {
            message: format!(
                "[{:#?}] Cell candidate removal - ({:?}, {:?}) -> {:?} Updated - {:?} - [Reason -> {:?}]",
                Utc::now().to_rfc3339(),
                index / dim,
                index % dim,
                val,
                self.cells[index].candidates,
                reason,
            ),
        });
    }

    fn replace_cell_candidates(
        &mut self,
        actor: usize,
        index: usize,
        candidates: &HashSet<usize>,
        reason: &str,
    ) {
        let dim = self.s_dim * self.s_dim;
        self.cells[index].candidates.clear();
        self.cells[index].candidates.extend(candidates.iter());
        self.post_update(Event {
            message: format!(
                "[{:#?}] Cell candidate replacement - ({:?}, {:?}) -> {:?} - [Reason -> {:?}]",
                Utc::now().to_rfc3339(),
                index / dim,
                index % dim,
                self.cells[index].candidates,
                reason,
            ),
        });
    }

    fn remove_num_candidate(
        &mut self,
        actor: usize,
        number: usize,
        cat: usize,
        index: usize,
        val: usize,
        reason: &str,
    ) {
        let dim = self.s_dim * self.s_dim;
        self.num_candidates[number - 1][cat][index].remove(&val);
        self.post_update(Event {
            message: format!(
                "[{:#?}] Number candidate removal - number {:?} cat - {:?} index - {:?} -> ({:?}, {:?}) Updated - {:?} - [Reason -> {:?}]",
                Utc::now().to_rfc3339(),
                number,
                cat,
                index,
                val / dim,
                val % dim,
                self.num_candidates[number - 1][cat][index],
                reason,
            ),
        });
    }

    fn post_update(&mut self, event: Event) {
        self.data_sender.send(event).unwrap();
        let cmd = self.control_receiver.recv().unwrap();
        if !cmd.eq("n") {
            self.state = 1;
        }
    }

    fn validate(&self) {
        let dim = self.s_dim * self.s_dim;
        for cat in 0..=2 {
            for index in 0..dim {
                let mut nums: HashSet<usize> = HashSet::new();
                for ci in find_candidate_cells(self.s_dim, cat, index) {
                    let val = self.cells[ci].val;
                    if val > 0 && nums.contains(&val) {
                        panic!("Invalid sudoku - {} (in {}) is already present for cat - {} index - {}", val, ci, cat, index);
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
        (s, complete)
    }
}

fn is_perfect_square(n: usize) -> bool {
    let sqrt_n = (n as f64).sqrt() as usize;
    let squared = Wrapping(sqrt_n) * Wrapping(sqrt_n);
    squared.0 == n
}

fn parse(buf: &str, data_sender: SyncSender<Event>, control_receiver: Receiver<String>) -> Board {
    let parse_cell = |cv: char| -> usize {
        if cv == '.' {
            0
        } else {
            cv.to_digit(10).unwrap() as usize
        }
    };
    let cell_values: Vec<usize> = buf.chars().map(parse_cell).collect();
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
        data_sender,
        control_receiver,
        num_candidates: vec![vec![vec![HashSet::new(); dim]; 3]; dim],
        logs: vec![],
        candidate_mask: HashMap::new(),
        state: 0,
    }
}

fn handle_cell(board_arc_mutex: Arc<Mutex<Board>>, s_dim: usize, index: usize) {
    let dim = s_dim * s_dim;
    let dcells = dependency_cells(s_dim, index);
    loop {
        thread::sleep(Duration::from_millis(HANDLE_DELAY));
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
                if val > 0 && board.cells[index].candidates.contains(&val) {
                    board.remove_cell_candidate(
                        0,
                        index,
                        val,
                        &format!(
                            "HandleCell: Dependency cell ({:?}. {:?}) has this value.",
                            di / dim,
                            di % dim
                        ),
                    );
                }
            }
            let candidates: Vec<&usize> = board.cells[index]
                .candidates
                .iter()
                .filter(|x| **x > usize::MIN)
                .collect();
            if candidates.iter().count() == 1 {
                let candidate = candidates.iter().next().unwrap();
                board.set(
                    0,
                    index,
                    **candidate,
                    &format!("HandleCell: Single valid candidate left - {:?}", candidates),
                );
                break;
            }
            if board.state > 0 {
                break;
            }
        }
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
    number: usize,
    cat: usize,
    index: usize,
) {
    let dim = s_dim * s_dim;
    let mut mutex_guard = board_arc_mutex.lock().unwrap();
    let board = mutex_guard.deref_mut();
    board.num_candidates[number - 1][cat][index] = find_candidate_cells(s_dim, cat, index);
    drop(board);
    drop(mutex_guard);
    loop {
        thread::sleep(Duration::from_millis(HANDLE_DELAY));
        {
            let mut mutex_guard = match board_arc_mutex.lock() {
                Ok(mg) => mg,
                Err(e) => {
                    println!("Error in getting lock in handle_number - {number} - {e}");
                    return;
                }
            };
            let board = mutex_guard.deref_mut();
            let mut candidates = board.num_candidates[number - 1][cat][index]
                .iter()
                .map(|x| *x)
                .collect::<Vec<usize>>();
            candidates.sort();
            if candidates.len() <= 1 {
                if candidates.len() == 1 {
                    let candidate: usize = candidates[0];
                    board.set(
                        1,
                        candidate,
                        number,
                        &format!(
                            "HandleNumber: Single valid candidate left - ({:?}, {:?}) for cat - {:?} and index - {:?} -> candidates - {:?}",
                            candidate / dim,
                            candidate % dim,
                            cat,
                            index,
                            candidates,
                        ),
                    );
                }
                break;
            }
            for candidate in candidates.iter() {
                if !board.cells[*candidate].candidates.contains(&number) {
                    board.remove_num_candidate(1, number, cat, index, *candidate, &format!(
                        "HandleNumber: Cell - ({:?}, {:?}) does not have number {:?} in its candidates - {:?}",
                        *candidate / dim,
                        *candidate % dim,
                        number,
                        board.cells[*candidate].candidates,
                    ));
                }
            }
            let mask_key = candidates
                .iter()
                .map(|x| (*x).to_string())
                .collect::<Vec<String>>()
                .join(",");
            let mask_numbers = board
                .candidate_mask
                .entry(mask_key)
                .or_insert(HashSet::new());
            mask_numbers.insert(number);
            let mask_numbers = mask_numbers.clone();
            drop(board);
            drop(mutex_guard);
            let mut mutex_guard = match board_arc_mutex.lock() {
                Ok(mg) => mg,
                Err(e) => {
                    println!("Error in getting lock in handle_number - {number} - {e}");
                    return;
                }
            };
            let board = mutex_guard.deref_mut();
            if candidates.iter().count() == mask_numbers.iter().count() {
                for ci in candidates.iter() {
                    board.replace_cell_candidates(
                        1,
                        *ci,
                        &mask_numbers,
                        &format!(
                            "Cells {:?} and candidates {:?} match size.",
                            candidates, mask_numbers
                        ),
                    );
                }
            }

            let row = candidates
                .iter()
                .map(|c| *c / dim)
                .reduce(|acc, e| if acc == e { acc } else { dim + 1 })
                .unwrap_or(dim + 1);
            if row != dim + 1 {
                let candidate_cells = find_candidate_cells(s_dim, 1, row);
                for candidate_cell in candidate_cells {
                    if !candidates.contains(&&candidate_cell)
                        && board.cells[candidate_cell].candidates.contains(&number)
                    {
                        board.remove_cell_candidate(
                            1,
                            candidate_cell,
                            number,
                            &format!(
                                "HandleNumber: Number {:?} has candidates {:?} which is in same row as cell {:?}.",
                                number,
                                candidates,
                                candidate_cell,
                            ),
                        );
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
                        board.remove_cell_candidate(
                            1,
                            candidate,
                            number,
                            &format!(
                                "HandleNumber: Number {:?} has candidates {:?} which is in same col as cell {:?}.",
                                number,
                                candidates,
                                candidate,
                            ),
                        );
                    }
                }
            }
            if board.state > 0 {
                break;
            }
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
