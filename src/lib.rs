//! Look at this [Link](std::iter::Iter)

#[macro_use]
extern crate pest_derive;

use terminal_size;

pub mod interpreter;
pub mod parser;
pub mod problem;
pub mod structure;

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

pub fn compile(sources: &SourceSet) -> Result<CompileResult, String> {
    // TODO: Handle multiple sources.
    // TODO: Pass name of source to parser.
    let mut ast_result = parser::parse(sources.borrow_sources()[1].1).expect("TODO Friendly error");

    let mut program = match parser::convert_ast_to_structure(&mut ast_result) {
        Result::Ok(program) => program,
        Result::Err(err) => {
            let width = terminal_size::terminal_size().map(|size| (size.0).0 as usize);
            return Result::Err(format!(
                "Compilation failed during structuring phase.\n\n{}",
                err.format(width, sources)
            ));
        }
    };

    let entry_point = program.get_entry_point();

    let new_entry_point = match structure::resolve_scope(&mut program, entry_point) {
        Result::Ok(new_root) => new_root,
        Result::Err(err) => {
            let width = terminal_size::terminal_size().map(|size| (size.0).0 as usize);
            return Result::Err(format!(
                "Compilation failed during resolving phase.\n\n{}",
                err.format(width, sources)
            ));
        }
    };
    program.set_entry_point(new_entry_point);

    Result::Ok(CompileResult {
        program: program,
    })
}
