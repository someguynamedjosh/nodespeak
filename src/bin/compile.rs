extern crate waveguide;

use std::fs;

fn main() {
    let code = fs::read_to_string("examples/structure.wg").expect("Cannot read structure.wg.");
    match waveguide::compile(&code) {
        Result::Ok(_program) => println!("Compilation completed sucessfully."),
        Result::Err(err) => eprintln!("{}", err)
    }
}