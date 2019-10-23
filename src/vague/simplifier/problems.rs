use crate::problem::*;
use crate::vague::structure::{DataType, KnownData};
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
            "Incorrect Type\nThe highlighted expression should simplify to a function because it ",
            "is being used in a function call. However, it simplifies to a different data type.",
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

pub fn array_index_not_int(
    index: FilePosition,
    index_type: &DataType,
    expression: FilePosition,
) -> CompileProblem {
    CompileProblem::from_descriptors(vec![
        ProblemDescriptor::new(
            index,
            Error,
            &format!(
                "Array Index Not Int\nExpected an integer, got a {:?}:",
                index_type
            ),
        ),
        ProblemDescriptor::new(
            expression,
            Hint,
            "Encountered while simplifying this expression:",
        ),
    ])
}

pub fn no_bct(
    expression: FilePosition,
    op1: FilePosition,
    op1_type: &DataType,
    op2: FilePosition,
    op2_type: &DataType,
) -> CompileProblem {
    CompileProblem::from_descriptors(vec![
        ProblemDescriptor::new(
            expression,
            Error,
            concat!(
                "No Biggest Common Type\nCannot determine what data type the result of the ",
                "highlighted expression will have:"
            ),
        ),
        ProblemDescriptor::new(
            op1,
            Hint,
            &format!("The first operand has data type {:?}:", op1_type),
        ),
        ProblemDescriptor::new(
            op2,
            Hint,
            &format!("But the second operand has data type {:?}:", op2_type),
        ),
    ])
}
