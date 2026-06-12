#![forbid(unsafe_code)]
use std::env;
use std::fs;
use std::io::{self, Read};

fn main() {
    let args: Vec<String> = env::args().collect();
    let mut input = String::new();

    if args.len() > 1 {
        // Read from file
        input = fs::read_to_string(&args[1]).expect("Failed to read file");
    } else {
        // Read from stdin
        io::stdin()
            .read_to_string(&mut input)
            .expect("Failed to read from stdin");
    }

    let output = marked_rs::parse(&input);
    print!("{}", output);
}
