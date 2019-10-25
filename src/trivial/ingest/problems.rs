use crate::problem::*;
use crate::vague::structure::{DataType, KnownData};
use ProblemType::Error;
use ProblemType::Hint;

pub fn array_size_not_int(size_pos: FilePosition, size_value: &KnownData) -> CompileProblem {
    CompileProblem::from_descriptors(vec![ProblemDescriptor::new(
        size_pos,
        Error,
        &format!(
            "Array Size Not Int\nExpected an integer, got {:?}:",
            size_value
        ),
    )])
}

pub fn vague_array_size(size_pos: FilePosition) -> CompileProblem {
    CompileProblem::from_descriptors(vec![ProblemDescriptor::new(
        size_pos,
        Error,
        "Vague Array Size\nCould not determine the value of this expression at compile time:",
    )])
}

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
            "The above expression being void makes this highlighted expression invalid:"
        )
    ])
}
