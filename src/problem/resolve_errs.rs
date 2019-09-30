use super::objects::*;
use ProblemType::Error;
use ProblemType::Hint;

pub fn wrong_number_of_inputs(
    func_call_pos: FilePosition,
    header_pos: FilePosition,
    provided: usize,
    expected: usize,
) -> CompileProblem {
    CompileProblem::from_descriptors(vec![
        ProblemDescriptor::new(
            func_call_pos,
            Error,
            &format!(
                concat!(
                    "Wrong Number Of Inputs\nThis function call has {} input ",
                    "arguments but the function it is calling has {} input ",
                    "parameters.",
                ),
                provided, expected,
            ),
        ),
        ProblemDescriptor::new(
            header_pos,
            Hint,
            concat!("The header of the function being called is as follows:"),
        ),
    ])
}

pub fn wrong_number_of_outputs(
    func_call_pos: FilePosition,
    header_pos: FilePosition,
    provided: usize,
    expected: usize,
) -> CompileProblem {
    CompileProblem::from_descriptors(vec![
        ProblemDescriptor::new(
            func_call_pos,
            Error,
            &format!(
                concat!(
                    "Wrong Number Of Outputs\nThis function call has {} output ",
                    "arguments but the function it is calling has {} output ",
                    "parameters.",
                ),
                provided, expected,
            ),
        ),
        ProblemDescriptor::new(
            header_pos,
            Hint,
            concat!("The header of the function being called is as follows:"),
        ),
    ])
}

pub fn vague_function(func_call_pos: FilePosition, expr_pos: FilePosition) -> CompileProblem {
    CompileProblem::from_descriptors(vec![
        ProblemDescriptor::new(
            func_call_pos,
            Error,
            concat!(
                "Vague Function\nCannot determine what function is ",
                "being called here:"
            ),
        ),
        ProblemDescriptor::new(
            expr_pos,
            Hint,
            concat!(
                "The highlighted expression must have a value that can be determined at compile ",
                "time, but it relies on values that will only be known at run time:"
            ),
        ),
    ])
}

pub fn not_function(expr_pos: FilePosition) -> CompileProblem {
    CompileProblem::from_descriptors(vec![ProblemDescriptor::new(
        expr_pos,
        Error,
        concat!(
            "Incorrect Type\nThe highlighted expression should resolve to a function because it ",
            "is being used in a function call. However, it resolves to a different data type.",
        ),
    )])
}

pub fn guaranteed_assert(assert_pos: FilePosition) -> CompileProblem {
    CompileProblem::from_descriptors(vec![ProblemDescriptor::new(
        assert_pos,
        Error,
        "Assert Guranteed To Fail",
    )])
}

pub fn illegal_inflation(source_pos: FilePosition, target_pos: FilePosition) -> CompileProblem {
    CompileProblem::from_descriptors(vec![
        ProblemDescriptor::new(
            source_pos,
            Error,
            "Illegal Inflation\nCannot inflate this value:",
        ),
        ProblemDescriptor::new(
            target_pos,
            Hint,
            "The value cannot be inflated to fit this target:",
        ),
    ])
}
