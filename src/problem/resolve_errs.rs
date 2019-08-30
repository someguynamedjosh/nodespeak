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
