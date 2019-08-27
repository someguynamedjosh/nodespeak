extern crate waveguide;

use std::env;
use std::fs;
use std::process;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: compile [path to file]");
        process::exit(64);
    } else {
        let code = match fs::read_to_string(&args[1]) {
            Result::Ok(content) => content,
            Result::Err(_err) => {
                eprintln!("Could not read from {}", &args[1]);
                process::exit(74);
            }
        };
        match waveguide::compile(&code) {
            Result::Ok(_program) => {
                println!("Compilation completed sucessfully.");
            }
            Result::Err(err) => {
                eprintln!("{}", err);
                process::exit(101);
            }
        }
    }
}
