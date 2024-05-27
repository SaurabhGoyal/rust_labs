const GRID_HEIGHT: usize = 20;
const GRID_WIDTH: usize = 100;
const BAT_LENGTH: usize = 10;

#[derive(Clone)]
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
            State::Bat => '-',
        }
    }
}

type Index = (usize, usize);

#[derive(Clone)]
struct Cell {
    index: Index,
    state: State,
}

struct Ball {
    index: Index,
    direction: u8,
    speed: u8,
}

struct Bat {
    index: Index,
    length: usize,
}

struct Game {
    grid: Vec<Vec<Cell>>,
    ball: Ball,
    bat: Bat,
}

impl Game {
    fn new() -> Self {
        let mut cells: Vec<Vec<Cell>> = Vec::with_capacity(GRID_HEIGHT);
        let mut ball_index: Index = (GRID_HEIGHT, GRID_WIDTH);
        let mut bat_index: Index = (GRID_HEIGHT, GRID_WIDTH);
        for i in 0..GRID_HEIGHT {
            let mut row = Vec::with_capacity(GRID_WIDTH);
            for j in 0..=GRID_WIDTH {
                let mut state = State::Empty;
                if i >= 3 && i <= 5 {
                    state = State::Brick
                } else if i == GRID_HEIGHT - 2 && j == GRID_WIDTH / 2 {
                    state = State::Ball;
                    ball_index = (i, j);
                } else if i == GRID_HEIGHT - 1
                    && j >= (GRID_WIDTH / 2 - BAT_LENGTH / 2)
                    && j <= (GRID_WIDTH / 2 + BAT_LENGTH / 2)
                {
                    state = State::Bat;
                    bat_index = (i, (GRID_WIDTH / 2 - BAT_LENGTH / 2));
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
                direction: 7,
                speed: 1,
            },
            bat: Bat {
                index: bat_index,
                length: BAT_LENGTH,
            },
        }
    }

    fn render(&self) -> String {
        let mut s = String::new();
        for i in 0..GRID_HEIGHT {
            for j in 0..=GRID_WIDTH {
                s.push(self.grid[i][j].state.char());
            }
            s.push('\n')
        }
        s
    }
}

fn main() {
    let g = Game::new();
    println!("{}", g.render());
}
