//! Look at this [Link](std::iter::Iter)

#[macro_use]
extern crate pest_derive;

use terminal_size;

pub mod parser;
pub mod problem;
pub mod structure;

pub struct CompileResult {
    pub program: structure::Program,
    pub root_scope: structure::ScopeId,
}

pub fn compile(source: &str) -> Result<CompileResult, String> {
    let mut ast_result = parser::parse(source).expect("TODO Friendly error");
    let mut program = match parser::convert_ast_to_structure(&mut ast_result) {
        Result::Ok(program) => program,
        Result::Err(err) => {
            let width = terminal_size::terminal_size().map(|size| (size.0).0 as usize);
            return Result::Err(format!(
                "Compilation failed during structuring phase.\n\n{}",
                err.format(width, source)
            ));
        }
    };
    let root_scope = program.get_root_scope();
    let new_root = structure::resolve_scope(&mut program, root_scope);
    Result::Ok(CompileResult {
        program: program,
        root_scope: new_root,
    })
}
