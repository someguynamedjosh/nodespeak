//! Look at this [Link](std::iter::Iter)

#[macro_use]
extern crate pest_derive;

use terminal_size;

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
    pub root_scope: structure::ScopeId,
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
    let root_scope = program.get_root_scope();
    let new_root = structure::resolve_scope(&mut program, root_scope);
    Result::Ok(CompileResult {
        program: program,
        root_scope: new_root,
    })
}
