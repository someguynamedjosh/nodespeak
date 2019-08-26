#[macro_use]
extern crate pest_derive;

use std::fs;

mod parser;
mod problem;
mod vague;

fn main() {
    let code = fs::read_to_string("examples/structure.wg").expect("Cannot read structure.wg.");
    let mut ast_result = parser::parse(&code).expect("Cannot parse structure.wg.");
    let mut vague_program = parser::convert_ast_to_vague(&mut ast_result);
    let root_scope = vague_program.get_root_scope();
    let resolved_root = vague::resolve_scope(&mut vague_program, root_scope);
    // The program includes both the vague and resolved scopes.
    // println!("{:#?}", vague_program);
}
