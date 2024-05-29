use std::{
    env,
    process::{exit, Command},
};

const CD_CMD: &str = "cd";
const EXIT_CMD: &str = "exit";
const PATH_CMD: &str = "path";

#[derive(Debug)]
pub struct Config {
    pub prompt: String,
    pub paths: Vec<String>,
}

pub fn parse_cmd(input: &str) -> Option<Command> {
    dbg!(input);
    let input = input.trim();
    if input.is_empty() {
        return None;
    }
    let mut parts = input.split_whitespace().map(|x| x.trim());
    let bin = parts.next().unwrap();
    let mut cmd = Command::new(bin);
    cmd.args(parts);
    Some(cmd)
}

pub fn execute(config: &mut Config, mut command: Command) {
    dbg!(&command);
    match command.get_program().to_str().unwrap() {
        CD_CMD => {
            let dir = command.get_args().next().unwrap();
            env::set_current_dir(dir).expect("error in cd");
        }
        PATH_CMD => {
            config.paths = command
                .get_args()
                .filter_map(|s| s.to_str())
                .map(|s| s.to_string())
                .collect();
        }
        EXIT_CMD => {
            exit(0);
        }
        _ => {
            let mut child = command
                .env("PATH", config.paths.join(":"))
                .spawn()
                .expect("error in spawn");
            child.wait().unwrap();
        }
    }
}
