mod objects;
mod resolve_errs;
mod runtime;
mod structure_errs;

pub use objects::*;
pub use resolve_errs::*;
pub use runtime::*;
pub use structure_errs::*;

pub fn bad_syntax(pos: FilePosition, message: String) -> CompileProblem {
    CompileProblem::from_descriptors(vec![ProblemDescriptor::new(
        pos,
        ProblemType::Error,
        &format!("Bad Syntax\n{}", message),
    )])
}
