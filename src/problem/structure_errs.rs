use super::objects::*;
use ProblemType::Error;
use ProblemType::Hint;

pub fn no_entity_with_name(pos: FilePosition) -> CompileProblem {
    CompileProblem::from_descriptors(vec![ProblemDescriptor::new(
        pos,
        Error,
        concat!(
            "Invalid Entity Name\nThere is no function, variable, or data type visible in this ",
            "scope with the specified name.",
        ),
    )])
}

pub fn return_from_root(pos: FilePosition) -> CompileProblem {
    CompileProblem::from_descriptors(vec![ProblemDescriptor::new(
        pos,
        Error,
        concat!(
            "Return Outside Function\nReturn statements can only be used inside of function ",
            "definitions. The code snippet below was understood to be a part of the root scope ",
            "of the file.",
        ),
    )])
}

pub fn extra_return_value(
    statement_pos: FilePosition,
    extra_pos: FilePosition,
    function_header: FilePosition,
    expected_num_values: usize,
) -> CompileProblem {
    CompileProblem::from_descriptors(vec![
        ProblemDescriptor::new(
            statement_pos,
            Error,
            &format!(
                concat!(
                    "Too Many Return Values\nThe highlighted return statement specifies too many ",
                    "values. The function it is a part of only has {} output values.",
                ),
                expected_num_values
            ),
        ),
        ProblemDescriptor::new(
            extra_pos,
            Hint,
            concat!("This value has no corresponding output."),
        ),
        ProblemDescriptor::new(
            function_header,
            Hint,
            concat!("The header of the function is as follows:"),
        ),
    ])
}

pub fn missing_return_values(
    statement_pos: FilePosition,
    function_header: FilePosition,
    expected_num_values: usize,
    actual_num_values: usize,
) -> CompileProblem {
    CompileProblem::from_descriptors(vec![
        ProblemDescriptor::new(
            statement_pos,
            Error,
            &format!(
                concat!(
                    "Not Enough Return Values\nReturn statements must either specify values for ",
                    "every output or specify no outputs. The highlighted return statement ",
                    "specifies values for {} outputs, but the function it is a part of has {} ",
                    "outputs."
                ),
                actual_num_values, expected_num_values,
            ),
        ),
        ProblemDescriptor::new(
            function_header,
            Hint,
            concat!("The header of the function is as follows:"),
        ),
    ])
}

pub fn missing_inline_return(
    func_call_pos: FilePosition,
    output_list_pos: FilePosition,
) -> CompileProblem {
    CompileProblem::from_descriptors(vec![
        ProblemDescriptor::new(
            func_call_pos,
            Error,
            concat!(
                "Missing Inline Return\nThe position of the highlighted function call requires it ",
                "to have an inline output so that the output can be used in an expression or ",
                "statement. However, there is no inline output argument in the output list.",
            ),
        ),
        ProblemDescriptor::new(
            output_list_pos,
            Hint,
            concat!(
                "Try replacing one of the output arguments with the keyword 'inline'. If the ",
                "function being called only has one output, you can also delete the output list ",
                "entirely to automatically make the only output inline."
            ),
        ),
    ])
}

pub fn io_inside_function(declaration_pos: FilePosition) -> CompileProblem {
    CompileProblem::from_descriptors(vec![ProblemDescriptor::new(
        declaration_pos,
        Error,
        concat!(
            "I/O Inside Function\nInput and output variables cannot be declared inside ",
            "functions. They can only be declared in the root scope, I.E. outside of functions."
        ),
    )])
}

pub fn hint_encountered_while_parsing(
    context_description: &str,
    assignment_pos: FilePosition,
    error: &mut CompileProblem,
) {
    error.add_descriptor(ProblemDescriptor::new(
        assignment_pos,
        Hint,
        &format!("Error encountered while parsing {}:", context_description),
    ));
}
