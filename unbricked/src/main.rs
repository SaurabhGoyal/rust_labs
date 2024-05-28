use console::Term;
use std::{
    io::{self, Write as _},
    process::exit,
    sync::{Arc, Mutex},
    thread::{sleep, spawn},
    time::Duration,
};

const REFRESH_RATE_MS: usize = 50;
const GRID_HEIGHT: i8 = 30;
const GRID_WIDTH: i8 = 100;
const BAT_LENGTH: usize = 16;

#[derive(PartialEq, Eq, Clone)]
enum State {
    Brick,
    Ball,
    Empty,
    Bat,
}

impl State {
    fn char(&self) -> char {
        match self {
            State::Brick => 'X',
            State::Ball => 'O',
            State::Empty => ' ',
            State::Bat => 'E',
        }
    }
}

type Pair = (i8, i8);

#[derive(Clone)]
struct Cell {
    index: Pair,
    state: State,
}

struct Ball {
    index: Pair,
    direction: Pair,
    speed: u8,
}

struct Bat {
    index: Pair,
    length: usize,
}

struct Game {
    grid: Vec<Vec<Cell>>,
    ball: Ball,
    bat: Bat,
    last_cmd: String,
}

impl Game {
    fn new() -> Self {
        let mut cells: Vec<Vec<Cell>> = Vec::with_capacity(GRID_HEIGHT as usize);
        let mut ball_index: Pair = (-1, -1);
        let mut bat_index: Pair = (-1, -1);
        let bat_j = GRID_WIDTH / 2 - BAT_LENGTH as i8 / 2;
        for i in 0..GRID_HEIGHT {
            let mut row = Vec::with_capacity(GRID_WIDTH as usize);
            for j in 0..GRID_WIDTH {
                let mut state = State::Empty;
                if i >= 3 && i <= 5 {
                    state = State::Brick
                } else if i == GRID_HEIGHT - 2 && j == GRID_WIDTH / 2 {
                    state = State::Ball;
                    ball_index = (i, j);
                } else if i == GRID_HEIGHT - 1 && j >= bat_j && j < bat_j + BAT_LENGTH as i8 {
                    state = State::Bat;
                    bat_index = (i, (GRID_WIDTH / 2 - BAT_LENGTH as i8 / 2));
                }
                row.push(Cell {
                    index: (i, j),
                    state,
                })
            }
            cells.push(row);
        }
        Game {
            grid: cells,
            ball: Ball {
                index: ball_index,
                direction: (-1, -1),
                speed: 1,
            },
            bat: Bat {
                index: bat_index,
                length: BAT_LENGTH,
            },
            last_cmd: String::new(),
        }
    }

    fn set_ball(&mut self, index: Pair, speed: u8, direction: Pair) {
        let (i, j) = self.ball.index;
        let (ni, nj) = (index.0, index.1);
        self.grid[i as usize][j as usize].state = State::Empty;
        self.grid[ni as usize][nj as usize].state = State::Ball;
        self.ball.index = (ni, nj);
        self.ball.speed = speed;
        self.ball.direction = direction;
    }

    fn set_bat(&mut self, index: Pair) {
        let (i, j) = self.bat.index;
        let (ni, nj) = (index.0, index.1);
        if ni < 0 || ni >= GRID_HEIGHT || nj < 0 || nj >= GRID_WIDTH - BAT_LENGTH as i8 {
            return;
        }
        for k in j..(j + BAT_LENGTH as i8) {
            self.grid[i as usize][k as usize].state = State::Empty;
        }
        for k in nj..(nj + BAT_LENGTH as i8) {
            self.grid[ni as usize][k as usize].state = State::Bat;
        }
        self.bat.index = (ni, nj);
    }

    fn next(&mut self) {
        if self.ball.speed == 0 {
            return;
        }
        let (i, j) = self.ball.index;
        let (ni, nj) = (i + self.ball.direction.0, j + self.ball.direction.1);
        let is_wall = || ni < 0 || ni >= GRID_HEIGHT || nj < 0 || nj >= GRID_WIDTH;
        let is_brick = || self.grid[ni as usize][nj as usize].state == State::Brick;
        let is_bat = || self.grid[ni as usize][nj as usize].state == State::Bat;
        if is_wall() {
            if ni < 0 {
                self.ball.direction = (-1 * self.ball.direction.0, self.ball.direction.1);
            }
            if ni >= GRID_HEIGHT {
                self.set_ball((GRID_HEIGHT - 2, GRID_WIDTH / 2), 0, (-1, -1));
                self.set_bat((GRID_HEIGHT - 1, GRID_WIDTH / 2 - BAT_LENGTH as i8 / 2));
            }
            if nj < 0 || nj >= GRID_WIDTH {
                self.ball.direction = (self.ball.direction.0, -1 * self.ball.direction.1);
            }
            return;
        }
        if is_brick() {
            if self.grid[i as usize][nj as usize].state != State::Brick {
                self.ball.direction = (-1 * self.ball.direction.0, self.ball.direction.1);
            }
            if self.grid[ni as usize][j as usize].state != State::Brick {
                self.ball.direction = (self.ball.direction.0, -1 * self.ball.direction.1);
            }
            self.grid[ni as usize][nj as usize].state = State::Empty;
            return;
        }
        if is_bat() {
            if self.grid[i as usize][nj as usize].state != State::Bat {
                self.ball.direction = (-1 * self.ball.direction.0, self.ball.direction.1);
            }
            if self.grid[ni as usize][j as usize].state != State::Bat {
                self.ball.direction = (self.ball.direction.0, -1 * self.ball.direction.1);
            }
            return;
        }
        self.grid[i as usize][j as usize].state = State::Empty;
        self.grid[ni as usize][nj as usize].state = State::Ball;
        self.ball.index = (ni, nj);
    }

    fn view(&self) -> String {
        let mut s = String::new();
        for i in 0..GRID_HEIGHT {
            for j in 0..GRID_WIDTH {
                s.push(self.grid[i as usize][j as usize].state.char());
            }
            s.push('\n')
        }
        s.push_str(&self.last_cmd);
        s.push('\n');
        s
    }
}

fn clear_screen() {
    print!("{}[2J", 27 as char); // ANSI escape code to clear the screen
    print!("{}[1;1H", 27 as char); // ANSI escape code to move the cursor to the top-left corner
    io::stdout().flush().unwrap(); // Flush stdout to ensure screen is cleared immediately
}

fn main() {
    let mut g = Game::new();
    g.ball.speed = 1;
    let g = Arc::new(Mutex::new(g));
    let gc = Arc::clone(&g);
    let rh = spawn(move || render(gc));
    let gc = Arc::clone(&g);
    let rih = spawn(move || read_input(gc));
    rh.join().unwrap();
    rih.join().unwrap();
}

fn render(game: Arc<Mutex<Game>>) {
    loop {
        {
            let mut g = game.lock().unwrap();
            clear_screen();
            println!("{}", g.view());
            g.next();
        }
        sleep(Duration::from_millis(REFRESH_RATE_MS as u64));
    }
}

fn read_input(game: Arc<Mutex<Game>>) -> ! {
    let stdout = Term::buffered_stdout();
    loop {
        {
            if let Ok(character) = stdout.read_char() {
                let mut g = game.lock().unwrap();
                g.last_cmd = String::from(format!("User pressed {character}"));
                match character {
                    ' ' => {
                        let (ni, nj) = (g.ball.index.0, g.ball.index.1);
                        g.set_ball((ni, nj), 1, (-1, -1));
                    }
                    'a' => {
                        let (ni, nj) = (g.bat.index.0, g.bat.index.1 - 1);
                        g.set_bat((ni, nj));
                    }
                    'd' => {
                        let (ni, nj) = (g.bat.index.0, g.bat.index.1 + 1);
                        g.set_bat((ni, nj));
                    }
                    _ => exit(0),
                }
            }
        }
    }
}
