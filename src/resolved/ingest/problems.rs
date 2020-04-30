use super::DataType;
use crate::problem::*;
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
            "Encountered while resolving this expression:",
        ),
    ])
}

pub fn array_index_less_than_zero(
    index: FilePosition,
    value: i64,
    expression: FilePosition,
) -> CompileProblem {
    CompileProblem::from_descriptors(vec![
        ProblemDescriptor::new(
            index,
            Error,
            &format!(
                concat!(
                    "Array Index Less Than Zero\nThe value of the highlighted expression was ",
                    "computed to be {}:",
                ),
                value
            ),
        ),
        ProblemDescriptor::new(
            expression,
            Hint,
            "Encountered while resolving this expression:",
        ),
    ])
}

pub fn array_index_too_big(
    index: FilePosition,
    value: i64,
    arr_size: i64,
    expression: FilePosition,
) -> CompileProblem {
    CompileProblem::from_descriptors(vec![
        ProblemDescriptor::new(
            index,
            Error,
            &format!(
                concat!(
                    "Array Index Too Big\nThe value of the highlighted expression was ",
                    "computed to be {}, which is too big when indexing an array of size {}:",
                ),
                value,
                arr_size,
            ),
        ),
        ProblemDescriptor::new(
            expression,
            Hint,
            "Encountered while resolving this expression:",
        ),
    ])
}

pub fn array_size_not_positive(size: FilePosition, value: i64) -> CompileProblem {
    CompileProblem::from_descriptors(vec![ProblemDescriptor::new(
        size,
        Error,
        &format!(
            concat!(
                "Non-Positive Array Size\nThe highlighted expression resolves to {}, which is not ",
                "greater than zero."
            ),
            value
        ),
    )])
}

pub fn array_size_not_int(
    size: FilePosition,
    size_type: &crate::vague::structure::DataType,
) -> CompileProblem {
    CompileProblem::from_descriptors(vec![ProblemDescriptor::new(
        size,
        Error,
        &format!(
            "Array Size Not Int\nExpected an integer, got a {:?}:",
            size_type
        ),
    )])
}

pub fn array_size_not_resolved(size: FilePosition) -> CompileProblem {
    CompileProblem::from_descriptors(vec![ProblemDescriptor::new(
        size,
        Error,
        concat!(
            "Dynamic Array Size\nArray sizes must be specified at compile time. The following ",
            "expression can only be evaluated at runtime:"
        ),
    )])
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

pub fn mismatched_assign(
    expression: FilePosition,
    lhs: FilePosition,
    lhs_type: &DataType,
    rhs: FilePosition,
    rhs_type: &DataType,
) -> CompileProblem {
    CompileProblem::from_descriptors(vec![
        ProblemDescriptor::new(
            expression,
            Error,
            concat!(
                "Mismatched Datatype In Assignment\nCannot figure out how to assign the ",
                "right hand side of this statement to the left hand side:"
            ),
        ),
        ProblemDescriptor::new(
            rhs,
            Hint,
            &format!("The right hand side has data type {:?}:", rhs_type),
        ),
        ProblemDescriptor::new(
            lhs,
            Hint,
            &format!("But the left hand side has data type {:?}:", lhs_type),
        ),
    ])
}

pub fn value_not_run_time_compatible(value_pos: FilePosition, dtype: &DataType) -> CompileProblem {
    CompileProblem::from_descriptors(vec![ProblemDescriptor::new(
        value_pos,
        Error,
        &format!(
            concat!(
                "Value Not Run Time Compatible\nThe value of the highlighted expression was ",
                "calculated at compile time, but the way it is used requires it to be available ",
                "at run time. This is not possible as it yields a value of type {:?}."
            ),
            dtype
        ),
    )])
}

pub fn too_many_indexes(
    expr_pos: FilePosition,
    num_indexes: usize,
    max_indexes: usize,
    base_pos: FilePosition,
    base_type: &DataType,
) -> CompileProblem {
    CompileProblem::from_descriptors(vec![
        ProblemDescriptor::new(
            expr_pos,
            Error,
            &format!(
                concat!(
                    "Too Many Indexes\nThe highlighted expression is indexing a value {} times, ",
                    "but the value it is indexing can only be indexed at most {} times."
                ),
                num_indexes, max_indexes
            ),
        ),
        ProblemDescriptor::new(
            base_pos,
            Hint,
            &format!(
                concat!("The base of the expression has the data type {:?}"),
                base_type
            ),
        ),
    ])
}
