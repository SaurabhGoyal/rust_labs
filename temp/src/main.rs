use console::Term;

fn main() {
    let stdout = Term::buffered_stdout();

    'game_loop: loop {
        if let Ok(character) = stdout.read_char() {
            match character {
                'w' => println!("Up"),
                'a' => println!("Left"),
                's' => println!("Down"),
                'd' => println!("Right"),
                _ => break 'game_loop,
            }
        }
    }
}
