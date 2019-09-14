//! Look at this [Link](std::iter::Iter)

#[macro_use]
extern crate pest_derive;

use terminal_size;

pub mod interpreter;
pub mod parser;
pub mod problem;
pub mod structure;
pub mod util;

pub struct SourceSet<'a> {
    sources: Vec<(String, &'a str)>,
}

impl<'a> SourceSet<'a> {
    pub fn new<'s>() -> SourceSet<'s> {
        SourceSet {
            sources: vec![(
                "(internal code) builtins".to_owned(),
                structure::FAKE_BUILTIN_SOURCE,
            )],
        }
    }

    pub fn from_raw_string<'s>(name: &str, content: &'s str) -> SourceSet<'s> {
        let mut result = Self::new::<'s>();
        result.sources.push((name.to_owned(), content));
        result
    }

    pub fn borrow_sources(&self) -> &Vec<(String, &'a str)> {
        &self.sources
    }
}

pub struct CompileResult {
    pub program: structure::Program,
}

fn compile_impl(sources: &SourceSet) -> Result<CompileResult, problem::CompileProblem> {
    let mut ast_result = parser::parse(sources.borrow_sources()[1].1)?;
    let mut program = parser::convert_ast_to_structure(&mut ast_result)?;
    let entry_point = program.get_entry_point();
    let new_entry_point = interpreter::resolve_scope(&mut program, entry_point)?;
    program.set_entry_point(new_entry_point);
    Result::Ok(CompileResult { program: program })
}

pub fn compile(sources: &SourceSet) -> Result<CompileResult, String> {
    // TODO: Handle multiple sources.
    compile_impl(sources).map_err(|err| {
        let width = terminal_size::terminal_size().map(|size| (size.0).0 as usize);
        format!("Compilation failed.\n\n{}", err.format(width, sources))
    })
}

pub fn interpret(
    compiled_program: &mut structure::Program,
    inputs: Vec<structure::KnownData>,
    sources: &SourceSet,
) -> Result<Vec<structure::KnownData>, String> {
    interpreter::interpret_from_entry_point(compiled_program, inputs).map_err(|err| {
        let width = terminal_size::terminal_size().map(|size| (size.0).0 as usize);
        format!("Interpretation failed.\n\n{}", err.format(width, sources))
    })
}
