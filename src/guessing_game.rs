mod guessing_game;

use rand::{self, Rng};
use std::{cmp::Ordering, io};

pub fn guessing_game() {
    let secret_number = rand::thread_rng().gen_range(1..=100);
    loop {
        let buff = take_guess();
        match buff.cmp(&secret_number) {
            Ordering::Greater => println!("Your guess is larger."),
            Ordering::Less => println!("Your guess is smaller."),
            Ordering::Equal => {
                println!("Your guess is correct.");
                break;
            }
        }
    }
}

fn take_guess() -> u32 {
    println!("Enter a random number as your guess.");
    let mut buff = String::new();
    io::stdin()
        .read_line(&mut buff)
        .expect("error in read_line");
    let buff: u32 = buff
        .trim()
        .parse()
        .expect("Not a number (secret was {secret_number}).");
    return buff;
}
