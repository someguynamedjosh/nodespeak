use crate::problem::*;
use ProblemType::Error;
use ProblemType::Hint;

pub fn void_value(value_pos: FilePosition, use_pos: FilePosition) -> CompileProblem {
    CompileProblem::from_descriptors(vec![
        ProblemDescriptor::new(
            value_pos,
            Error,
            "Void Value\nThe highlighted expression does not return anything:",
        ),
        ProblemDescriptor::new(
            use_pos,
            Hint,
            "The above expression being void makes this highlighted expression invalid:",
        ),
    ])
}
