use std::process::Command;

const CD_CMD: &str = "cd";
const EXIT_CMD: &str = "exit";
const PATH_CMD: &str = "path";

#[derive(Debug)]
pub enum ShellError {
    UnknownCommand,
}

#[derive(Debug)]
pub struct Config {
    pub prompt: String,
    pub paths: Vec<String>,
}

pub fn parse_cmd(config: &Config, input: &str) -> Option<Command> {
    dbg!(input);
    let input = input.trim();
    if input.len() == 0 {
        return None;
    }
    let mut parts = input.split_whitespace().map(|x| x.trim());
    let bin = parts.next().unwrap();
    let mut cmd = Command::new(bin);
    cmd.env("PATH", config.paths.join(":"));
    cmd.args(parts);
    return Some(cmd);
}

pub fn execute(mut command: Command) {
    dbg!(&command);
    match command.get_program().to_str().unwrap() {
        CD_CMD => {}
        _ => {
            let mut child = command.spawn().expect("error in spawn");
            child.wait().unwrap();
        }
    }
}
