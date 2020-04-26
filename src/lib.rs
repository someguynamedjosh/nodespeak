//! Look at this [Link](std::iter::Iter)

#[macro_use]
extern crate pest_derive;

use terminal_size;

pub mod ast;
pub mod problem;
#[cfg(not(feature = "no-resolved"))]
pub mod resolved;
pub mod shared;
#[cfg(not(feature = "no-trivial"))]
pub mod trivial;
pub mod util;
#[cfg(not(feature = "no-vague"))]
pub mod vague;

pub struct SourceSet<'a> {
    sources: Vec<(String, &'a str)>,
}

impl<'a> SourceSet<'a> {
    #[cfg(feature = "no-vague")]
    pub fn new<'s>() -> SourceSet<'s> {
        SourceSet {
            sources: vec![(
                "placeholder".to_owned(),
                "placeholder"
            )],
        }
    }

    #[cfg(not(feature = "no-vague"))]
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

fn parse_impl<'a>(
    sources: &SourceSet<'a>,
) -> Result<ast::structure::Program<'a>, problem::CompileProblem> {
    ast::ingest(sources.borrow_sources()[1].1)
}

#[cfg(not(feature = "no-vague"))]
fn structure_impl(
    sources: &SourceSet,
) -> Result<vague::structure::Program, problem::CompileProblem> {
    let mut parsed = parse_impl(sources)?;
    vague::ingest(&mut parsed)
}

#[cfg(not(feature = "no-resolved"))]
fn resolve_impl(
    sources: &SourceSet,
) -> Result<resolved::structure::Program, problem::CompileProblem> {
    let mut program = structure_impl(sources)?;
    resolved::ingest(&mut program)
}

#[cfg(not(feature = "no-trivial"))]
fn trivialize_impl(
    sources: &SourceSet,
) -> Result<trivial::structure::Program, problem::CompileProblem> {
    let resolved = resolve_impl(sources)?;
    trivial::ingest(&resolved)
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

#[cfg(not(feature = "no-vague"))]
pub fn structure(sources: &SourceSet) -> Result<vague::structure::Program, String> {
    structure_impl(sources).map_err(error_map(sources))
}

#[cfg(not(feature = "no-resolved"))]
pub fn resolve(sources: &SourceSet) -> Result<resolved::structure::Program, String> {
    resolve_impl(sources).map_err(error_map(sources))
}

#[cfg(not(feature = "no-trivial"))]
pub fn trivialize(sources: &SourceSet) -> Result<trivial::structure::Program, String> {
    trivialize_impl(sources).map_err(error_map(sources))
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
