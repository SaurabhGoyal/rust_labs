use crate::tree::{BuildMethod, Config};
use regex::Regex;

const ARG_DEPTH_KEY: &str = "-d";
const ARG_EXCLUDE_KEY: &str = "-e";
const ARG_METHOD_KEY: &str = "-m";

impl Config {
    pub fn build_from_args(mut args: impl Iterator<Item = String>) -> Config {
        // Skip the process name arg.
        args.next().unwrap();
        let mut config = Config {
            path: args.next().unwrap(),
            depth_check: None,
            exclude_pattern: None,
            build_method: BuildMethod::SerialAsync,
        };
        loop {
            let item = args.next();
            if item.is_none() {
                break;
            }
            let item = item.unwrap();
            if !item.starts_with('-') {
                panic!("each arg must start with a hyphen");
            } else {
                match item.as_str() {
                    ARG_DEPTH_KEY => {
                        config.depth_check = Some(args.next().unwrap().parse::<u32>().unwrap());
                    }
                    ARG_EXCLUDE_KEY => {
                        let pattern = args.next().unwrap();
                        let _ = Regex::new(&pattern).unwrap();
                        config.exclude_pattern = Some(pattern);
                    }
                    ARG_METHOD_KEY => {
                        config.build_method = BuildMethod::from_str(args.next().unwrap().as_str());
                    }
                    _ => {
                        panic!("invalid arg")
                    }
                }
            }
        }
        config
    }
}
