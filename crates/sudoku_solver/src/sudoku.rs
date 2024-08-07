use std::{
    collections::{HashMap, HashSet},
    num::Wrapping,
    ops::DerefMut,
    sync::{
        mpsc::{sync_channel, Receiver, SyncSender},
        Arc, Mutex,
    },
    thread::{self, JoinHandle},
    time::Duration,
};

const HANDLE_DELAY: u64 = 200;

struct Cell {
    val: usize,
    fixed: bool,
    candidates: HashSet<usize>,
}

#[derive(Debug)]
pub struct Event {
    pub seq: usize,
    pub message: String,
}

#[derive(Debug)]
pub struct GameState {
    pub event: Event,
    pub repr: String,
    pub p_repr: String,
    pub complete: bool,
}

pub struct SudokuSolver {
    buf: String,
    board_arc_mutex: Arc<Mutex<Board>>,
    thread_handles: Vec<JoinHandle<()>>,
}

impl SudokuSolver {
    pub fn new(mut buf: String) -> (Self, SyncSender<String>, Receiver<GameState>) {
        // validate
        if buf.is_empty() {
            panic!("Empty string");
        }
        if buf.ends_with("\n") {
            buf.pop();
        }
        // initialise channels and board
        let (data_sender, data_receiver) = sync_channel::<GameState>(1);
        let (control_sender, control_receiver) = sync_channel::<String>(1);
        let board = Board::from(&buf, data_sender, control_receiver);

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
}

impl Drop for SudokuSolver {
    fn drop(&mut self) {
        println!("Dropping SS! Releasing resource");
        {
            let mut mutex_guard = self.board_arc_mutex.lock().unwrap();
            if mutex_guard.state == 0 {
                mutex_guard.state = 2;
            }
        }
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
    data_sender: SyncSender<GameState>,
    control_receiver: Receiver<String>,
    num_candidates: Vec<Vec<Vec<HashSet<usize>>>>,
    state: u8,
    event_logs: Vec<String>,
    last_event_seq: usize,
    candidate_mask: HashMap<String, HashSet<usize>>,
}

impl Board {
    fn from(
        buf: &str,
        data_sender: SyncSender<GameState>,
        control_receiver: Receiver<String>,
    ) -> Self {
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
            event_logs: vec![],
            last_event_seq: 0,
            candidate_mask: HashMap::new(),
            state: 0,
        }
    }

    fn set(&mut self, actor: usize, index: usize, val: usize, reason: &str) {
        if self.state > 0 {
            return;
        }
        if self.cells[index].val == val {
            return;
        }
        self.validate();
        let dim = self.s_dim * self.s_dim;
        self.cells[index].val = val;
        self.cells[index].candidates.clear();
        self.cells[index].candidates.insert(val);
        if self.cells.iter().filter(|c| c.val == 0).count() == 0 {
            self.state = 1;
        }
        self.post_update(format!(
            "[{:?}] Cell candidate finalisation - ({:?}, {:?}) -> {:?} - [Reason -> {:?}]",
            actor,
            index / dim,
            index % dim,
            val,
            reason,
        ));
    }

    fn remove_cell_candidate(&mut self, actor: usize, index: usize, val: usize, reason: &str) {
        if self.state > 0 {
            return;
        }
        if !self.cells[index].candidates.contains(&val) {
            return;
        }
        let dim = self.s_dim * self.s_dim;
        self.cells[index].candidates.remove(&val);
        self.post_update(format!(
            "[{:?}] Cell candidate removal - ({:?}, {:?}) -> {:?} Updated - {:?} - [Reason -> {:?}]",
            actor,
            index / dim,
            index % dim,
            val,
            self.cells[index].candidates,
            reason,
        ));
    }

    fn intersect_cell_candidates(
        &mut self,
        actor: usize,
        index: usize,
        candidates: &HashSet<usize>,
        reason: &str,
    ) {
        if self.state > 0 {
            return;
        }
        let discardable_candidates = self.cells[index]
            .candidates
            .iter()
            .copied()
            .filter(|c| !candidates.contains(c))
            .collect::<Vec<usize>>();
        if discardable_candidates.is_empty() {
            return;
        }
        let dim = self.s_dim * self.s_dim;
        for dc in discardable_candidates.iter() {
            self.cells[index].candidates.remove(dc);
        }
        self.post_update(format!(
            "[{:?}] Cell candidate intersection - ({:?}, {:?}) -> Based on grouping - {:?}, discarded - {:?}, updated - {:?} - [Reason -> {:?}]",
            actor,
            index / dim,
            index % dim,
            candidates,
            discardable_candidates,
            self.cells[index].candidates,
            reason,
        ));
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
        if self.state > 0 {
            return;
        }
        if !self.num_candidates[number - 1][cat][index].contains(&val) {
            return;
        }
        let dim = self.s_dim * self.s_dim;
        self.num_candidates[number - 1][cat][index].remove(&val);
        self.post_update(format!(
                "[{:?}] Number candidate removal - number - {:?} cat - {:?} index - {:?} -> ({:?}, {:?}) Updated - {:?} - [Reason -> {:?}]",
                actor,
                number,
                cat,
                index,
                val / dim,
                val % dim,
                self.num_candidates[number - 1][cat][index],
                reason,
            ));
    }

    fn post_update(&mut self, message: String) {
        self.event_logs.push(message.clone());
        self.last_event_seq += 1;
        // Send data, end game if no receiver available.
        if self
            .data_sender
            .send(GameState {
                event: Event {
                    message,
                    seq: self.last_event_seq,
                },
                repr: self.repr(),
                p_repr: self.pretty_repr(),
                complete: self.state == 1,
            })
            .is_err()
        {
            self.state = 2;
            return;
        }
        // Receiver command, end game if no sender available.
        if !self
            .control_receiver
            .recv()
            .unwrap_or("b".to_string())
            .eq("n")
        {
            self.state = 2;
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

    fn pretty_repr(&self) -> String {
        let dim = self.s_dim * self.s_dim;
        let line_sep_single = format!("\n{}\n", vec!["-"; 6 * dim + 1].join(""));
        let line_sep_double = format!("\n{}\n", vec!["⹀"; 6 * dim + 1].join(""));
        let mut s = String::new();
        s.push_str(&line_sep_double);
        for (i, cell) in self.cells.iter().enumerate() {
            let val_str = if cell.val == 0 {
                "     ".to_string()
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
        s
    }

    fn repr(&self) -> String {
        let mut s = String::new();
        for c in self.cells.iter() {
            s.push(if c.val > 0 {
                char::from_digit(c.val as u32, 10).unwrap()
            } else {
                '.'
            });
        }
        s
    }
}

fn is_perfect_square(n: usize) -> bool {
    let sqrt_n = (n as f64).sqrt() as usize;
    let squared = Wrapping(sqrt_n) * Wrapping(sqrt_n);
    squared.0 == n
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
                            "HandleCell: Dependency cell ({:?}, {:?}) has this value.",
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
            if candidates.len() == 1 {
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
                return;
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
    let mut board = board_arc_mutex.lock().unwrap();
    board.num_candidates[number - 1][cat][index] = find_candidate_cells(s_dim, cat, index);
    drop(board);
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
                .copied()
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
            let mut mask_numbers_clone = HashSet::new();
            mask_numbers.iter().for_each(|x| {
                mask_numbers_clone.insert(*x);
            });
            let mask_numbers = mask_numbers_clone;
            drop(mutex_guard);
            let mut mutex_guard = match board_arc_mutex.lock() {
                Ok(mg) => mg,
                Err(e) => {
                    println!("Error in getting lock in handle_number - {number} - {e}");
                    return;
                }
            };
            let board = mutex_guard.deref_mut();
            if mask_numbers.len() > 1 && candidates.len() == mask_numbers.len() {
                for ci in candidates.iter() {
                    board.intersect_cell_candidates(
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
                return;
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
