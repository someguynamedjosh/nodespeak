//! Look at this [Link](std::iter::Iter)

#[macro_use]
extern crate pest_derive;

use terminal_size;

pub mod ast;
#[cfg(not(feature = "no-llvmir"))]
pub mod llvm;
pub mod problem;
#[cfg(not(feature = "no-resolved"))]
pub mod resolved;
pub mod shared;
#[cfg(not(feature = "no-trivial"))]
pub mod trivial;
pub mod util;
#[cfg(not(feature = "no-vague"))]
pub mod vague;

pub struct SourceSet {
    sources: Vec<(String, String)>,
}

impl SourceSet {
    #[cfg(feature = "no-vague")]
    pub fn new() -> SourceSet {
        SourceSet {
            sources: vec![("placeholder".to_owned(), "placeholder")],
        }
    }

    #[cfg(not(feature = "no-vague"))]
    pub fn new() -> SourceSet {
        SourceSet {
            sources: vec![(
                "(internal code) builtins".to_owned(),
                vague::structure::FAKE_BUILTIN_SOURCE.to_owned(),
            )],
        }
    }

    pub fn add_item(&mut self, name: String, content: String) {
        self.sources.push((name, content));
    }

    pub fn add_item_from_file(&mut self, filename: String) -> std::io::Result<()> {
        let content = std::fs::read_to_string(&filename)?;
        self.sources.push((filename, content));
        Ok(())
    }

    pub fn borrow_sources(&self) -> &Vec<(String, String)> {
        &self.sources
    }
}

fn to_ast_impl<'a>(
    sources: &'a SourceSet,
) -> Result<ast::structure::Program<'a>, problem::CompileProblem> {
    ast::ingest(&sources.borrow_sources()[1].1)
}

#[cfg(not(feature = "no-vague"))]
fn to_vague_impl(
    sources: &SourceSet,
) -> Result<vague::structure::Program, problem::CompileProblem> {
    let mut parsed = to_ast_impl(sources)?;
    vague::ingest(&mut parsed)
}

#[cfg(not(feature = "no-resolved"))]
fn to_resolved_impl(
    sources: &SourceSet,
) -> Result<resolved::structure::Program, problem::CompileProblem> {
    let mut program = to_vague_impl(sources)?;
    resolved::ingest(&mut program)
}

#[cfg(not(feature = "no-trivial"))]
fn to_trivial_impl(
    sources: &SourceSet,
) -> Result<trivial::structure::Program, problem::CompileProblem> {
    let resolved = to_resolved_impl(sources)?;
    trivial::ingest(&resolved)
}

#[cfg(not(feature = "no-llvmir"))]
fn to_llvmir_impl(
    sources: &SourceSet,
) -> Result<llvm::structure::Program, problem::CompileProblem> {
    let trivial = to_trivial_impl(sources)?;
    Ok(llvm::ingest(&trivial))
}

fn error_map<'a>(sources: &'a SourceSet) -> impl Fn(problem::CompileProblem) -> String + 'a {
    move |err: problem::CompileProblem| -> String {
        let width = terminal_size::terminal_size().map(|size| (size.0).0 as usize);
        format!("Compilation failed.\n\n{}", err.format(width, sources))
    }
}

pub fn to_ast<'a>(sources: &'a SourceSet) -> Result<ast::structure::Program<'a>, String> {
    to_ast_impl(sources).map_err(error_map(sources))
}

#[cfg(not(feature = "no-vague"))]
pub fn to_vague(sources: &SourceSet) -> Result<vague::structure::Program, String> {
    to_vague_impl(sources).map_err(error_map(sources))
}

#[cfg(not(feature = "no-resolved"))]
pub fn to_resolved(sources: &SourceSet) -> Result<resolved::structure::Program, String> {
    to_resolved_impl(sources).map_err(error_map(sources))
}

#[cfg(not(feature = "no-trivial"))]
pub fn to_trivial(sources: &SourceSet) -> Result<trivial::structure::Program, String> {
    to_trivial_impl(sources).map_err(error_map(sources))
}

#[cfg(not(feature = "no-llvmir"))]
pub fn to_llvmir(sources: &SourceSet) -> Result<llvm::structure::Program, String> {
    to_llvmir_impl(sources).map_err(error_map(sources))
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
