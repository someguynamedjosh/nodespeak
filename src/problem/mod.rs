mod objects;
mod simplify_errs;
mod structure_errs;

pub use objects::*;
pub use simplify_errs::*;
pub use structure_errs::*;

pub fn bad_syntax(pos: FilePosition, message: String) -> CompileProblem {
    CompileProblem::from_descriptors(vec![ProblemDescriptor::new(
        pos,
        ProblemType::Error,
        &format!("Bad Syntax\n{}", message),
    )])
}
