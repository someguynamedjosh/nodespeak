//! Look at this [Link](std::iter::Iter)

#[macro_use]
extern crate pest_derive;

use terminal_size;

pub mod ast;
pub mod problem;
pub mod runtime;
pub mod trivial;
pub mod util;
pub mod vague;

pub struct SourceSet<'a> {
    sources: Vec<(String, &'a str)>,
}

impl<'a> SourceSet<'a> {
    pub fn new<'s>() -> SourceSet<'s> {
        SourceSet {
            sources: vec![(
                "(internal code) builtins".to_owned(),
                vague::structure::FAKE_BUILTIN_SOURCE,
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
    pub program: vague::structure::Program,
}

fn compile_impl(sources: &SourceSet) -> Result<CompileResult, problem::CompileProblem> {
    let mut ast_result = ast::ingest(sources.borrow_sources()[1].1)?;
    let mut program = vague::ingest(&mut ast_result)?;
    vague::simplify(&mut program)?;
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
    compiled_program: &mut vague::structure::Program,
    inputs: Vec<vague::structure::KnownData>,
    sources: &SourceSet,
) -> Result<Vec<vague::structure::KnownData>, String> {
    let entry_point = compiled_program.get_entry_point();
    let input_targets = compiled_program[entry_point].borrow_inputs().clone();
    for (source, target) in inputs.into_iter().zip(input_targets.into_iter()) {
        compiled_program[target].set_temporary_value(source);
    }
    vague::simplify(compiled_program).map_err(|err| {
        let width = terminal_size::terminal_size().map(|size| (size.0).0 as usize);
        format!("Compilation failed.\n\n{}", err.format(width, sources))
    })?;
    let entry_point = compiled_program.get_entry_point();
    let mut outputs = Vec::new();
    let output_sources = compiled_program[entry_point].borrow_outputs().clone();
    for source in output_sources.into_iter() {
        outputs.push(compiled_program[source].borrow_temporary_value().clone());
    }
    Result::Ok(outputs)
}
