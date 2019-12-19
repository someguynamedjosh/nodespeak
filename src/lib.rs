//! Look at this [Link](std::iter::Iter)

#[macro_use]
extern crate pest_derive;

use terminal_size;

pub mod ast;
pub mod native;
pub mod problem;
pub mod resolved;
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
    pub program: trivial::structure::Program,
}

fn parse_impl<'a>(
    sources: &SourceSet<'a>,
) -> Result<ast::structure::Program<'a>, problem::CompileProblem> {
    ast::ingest(sources.borrow_sources()[1].1)
}

fn structure_impl(
    sources: &SourceSet,
) -> Result<vague::structure::Program, problem::CompileProblem> {
    let mut parsed = parse_impl(sources)?;
    vague::ingest(&mut parsed)
}

fn resolve_impl(
    sources: &SourceSet,
) -> Result<resolved::structure::Program, problem::CompileProblem> {
    let mut program = structure_impl(sources)?;
    resolved::ingest(&mut program)
}

fn trivialize_impl(
    sources: &SourceSet,
) -> Result<trivial::structure::Program, problem::CompileProblem> {
    let resolved = resolve_impl(sources)?;
    trivial::ingest(&resolved)
}

fn compile_impl(sources: &SourceSet) -> Result<CompileResult, problem::CompileProblem> {
    let trivialized = trivialize_impl(sources)?;
    Result::Ok(CompileResult {
        program: trivialized,
    })
}

fn error_map<'a>(sources: &'a SourceSet) -> impl Fn(problem::CompileProblem) -> String + 'a {
    move |err: problem::CompileProblem| -> String {
        let width = terminal_size::terminal_size().map(|size| (size.0).0 as usize);
        format!("Compilation failed.\n\n{}", err.format(width, sources))
    }
}

pub fn parse<'a>(sources: &SourceSet<'a>) -> Result<ast::structure::Program<'a>, String> {
    parse_impl(sources).map_err(error_map(sources))
}

pub fn structure(sources: &SourceSet) -> Result<vague::structure::Program, String> {
    structure_impl(sources).map_err(error_map(sources))
}

pub fn resolve(sources: &SourceSet) -> Result<resolved::structure::Program, String> {
    resolve_impl(sources).map_err(error_map(sources))
}

pub fn trivialize(sources: &SourceSet) -> Result<trivial::structure::Program, String> {
    trivialize_impl(sources).map_err(error_map(sources))
}

pub fn compile(sources: &SourceSet) -> Result<CompileResult, String> {
    // TODO: Handle multiple sources.
    compile_impl(sources).map_err(error_map(sources))
}

// pub fn interpret(
//     compiled_program: &mut vague::structure::Program,
//     inputs: Vec<vague::structure::KnownData>,
//     sources: &SourceSet,
// ) -> Result<Vec<vague::structure::KnownData>, String> {
//     let entry_point = compiled_program.get_entry_point();
//     let input_targets = compiled_program[entry_point].borrow_inputs().clone();
//     for (source, target) in inputs.into_iter().zip(input_targets.into_iter()) {
//         compiled_program[target].set_temporary_value(source);
//     }
//     vague::resolve(compiled_program).map_err(|err| {
//         let width = terminal_size::terminal_size().map(|size| (size.0).0 as usize);
//         format!("Compilation failed.\n\n{}", err.format(width, sources))
//     })?;
//     let entry_point = compiled_program.get_entry_point();
//     let mut outputs = Vec::new();
//     let output_sources = compiled_program[entry_point].borrow_outputs().clone();
//     for source in output_sources.into_iter() {
//         outputs.push(compiled_program[source].borrow_temporary_value().clone());
//     }
//     Result::Ok(outputs)
// }
