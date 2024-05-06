use super::tree::CommandArgs;
use regex::Regex;

const ARG_DEPTH_KEY: &str = "-d";
const ARG_EXCLUDE_KEY: &str = "-e";

pub fn parse_command(args: &Vec<String>) -> CommandArgs {
    let mut cmd_args = CommandArgs {
        path: String::from(args.get(1).unwrap()),
        depth_check: None,
        exclude_pattern: None,
    };
    let mut i = 2;
    loop {
        let item = args.get(i);
        if item.is_none() {
            break;
        }
        let item = item.unwrap();
        if !item.starts_with("-") {
            panic!("each arg must start with a hyphen");
        } else {
            if item.eq(ARG_DEPTH_KEY) {
                i += 1;
                cmd_args.depth_check = Some(args.get(i).unwrap().parse::<u32>().unwrap());
            } else if item.eq(ARG_EXCLUDE_KEY) {
                i += 1;
                let pattern = args.get(i).unwrap();
                let _ = Regex::new(pattern).unwrap();
                cmd_args.exclude_pattern = Some(String::from(pattern));
            } else {
                panic!("invalid arg")
            }
            i += 1;
        }
    }
    return cmd_args;
}
