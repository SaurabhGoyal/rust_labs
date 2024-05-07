use super::tree::Config;
use regex::Regex;

const ARG_DEPTH_KEY: &str = "-d";
const ARG_EXCLUDE_KEY: &str = "-e";

impl Config {
    pub fn build_from_args(mut args: impl Iterator<Item = String>) -> Config {
        // Skip the process name arg.
        args.next().unwrap();
        let mut config = Config {
            path: args.next().unwrap(),
            depth_check: None,
            exclude_pattern: None,
        };
        loop {
            let item = args.next();
            if item.is_none() {
                break;
            }
            let item = item.unwrap();
            if !item.starts_with("-") {
                panic!("each arg must start with a hyphen");
            } else {
                if item.eq(ARG_DEPTH_KEY) {
                    config.depth_check = Some(args.next().unwrap().parse::<u32>().unwrap());
                } else if item.eq(ARG_EXCLUDE_KEY) {
                    let pattern = args.next().unwrap();
                    let _ = Regex::new(&pattern).unwrap();
                    config.exclude_pattern = Some(pattern);
                } else {
                    panic!("invalid arg")
                }
            }
        }
        return config;
    }
}
