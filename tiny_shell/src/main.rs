mod shell;
use std::{
    env,
    io::{self, Write},
};

const DEFAULT_PROMPT: &str = "tsh";
const DEFAULT_PATHS: [&str; 2] = ["/bin", "/usr/bin"];

fn main() {
    let mut config = shell::Config {
        prompt: String::from(DEFAULT_PROMPT),
        paths: DEFAULT_PATHS.iter().map(|s| s.to_string()).collect(),
    };
    loop {
        print!("{:#?}<{:#?}> ", config.prompt, env::current_dir().unwrap());
        io::stdout().flush().unwrap();
        let mut input = String::new();
        let read_bytes = io::stdin().read_line(&mut input).unwrap_or_else(|err| {
            println!("{:?}", err);
            0
        });
        if read_bytes == 0 {
            continue;
        }
        let res: Vec<()> = input
            .split("&")
            .map(|i| shell::parse_cmd(i))
            .filter(|c| c.is_some())
            .map(|c| c.unwrap())
            .map(|c| shell::execute(&mut config, c))
            .collect();
        dbg!(res);
    }
}
