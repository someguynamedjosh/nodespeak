#[macro_use]
extern crate pest_derive;

use std::fs;

mod parser;
mod vague;

fn main() {
    let code = fs::read_to_string("examples/arithmetic.wg")
        .expect("Cannot read arithmetic.wg.");
    let mut ast_result = parser::parse(&code)
        .expect("Cannot parse arithmetic.wg.");
    let vague_program = parser::convert_ast_to_vague(&mut ast_result);
    println!("{:#?}", vague_program);
}
