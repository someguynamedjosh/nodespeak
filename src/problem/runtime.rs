use super::objects::*;
use ProblemType::{Error, Hint};

pub fn assert_failed(pos: FilePosition) -> CompileProblem {
    CompileProblem::from_descriptors(vec![ProblemDescriptor::new(pos, Error, "Assert Failed")])
}

pub fn unknown_data(pos: FilePosition) -> CompileProblem {
    CompileProblem::from_descriptors(vec![ProblemDescriptor::new(
        pos,
        Error,
        concat!(
            "Unknown Data\nThe highlighted variable is being used in an expression even though it ",
            "has not had any data assigned to it. Ensure that the program always writes data to ",
            "this variable before it is used."
        ),
    )])
}

pub fn indexing_non_array(base: FilePosition, indexing: FilePosition) -> CompileProblem {
    CompileProblem::from_descriptors(vec![
        ProblemDescriptor::new(
            base,
            Error,
            concat!(
                "Indexing Non-Array\nThe following expression does not resolve to an array, but ",
                "it is being indexed as if it is one:"
            ),
        ),
        ProblemDescriptor::new(indexing, Hint, "Indexing occurs here:"),
    ])
}

pub fn index_not_int(pos: FilePosition) -> CompileProblem {
    CompileProblem::from_descriptors(vec![ProblemDescriptor::new(
        pos,
        Error,
        concat!(
            "Index Not Integer\nArray indexes must be integers, but the highlighted expression ",
            "does not resolve to an integer."
        ),
    )])
}

pub fn index_not_in_range(
    base: FilePosition,
    indexing: FilePosition,
    indexes: &Vec<usize>,
) -> CompileProblem {
    let mut index_description = "".to_owned();
    for index in indexes {
        index_description += &format!("[{}]", index);
    }
    CompileProblem::from_descriptors(vec![
        ProblemDescriptor::new(
            base,
            Error,
            &format!(
                concat!(
                    "Index Not In Range\nThe specified index {} is not in range of the ",
                    "highlighted value."
                ),
                index_description
            ),
        ),
        ProblemDescriptor::new(indexing, Hint, "Indexing occurs here:"),
    ])
}

pub fn assert_not_bool(argument: FilePosition, assert: FilePosition) -> CompileProblem {
    CompileProblem::from_descriptors(vec![
        ProblemDescriptor::new(
            argument,
            Error,
            concat!(
                "Incorrect Type\nThe value in an assert expression must be a boolean value, but ",
                "the highlighted expression does not resolve to a boolean value:"
            ),
        ),
        ProblemDescriptor::new(
            assert,
            Hint,
            concat!("Encountered while interpreting assertion:",),
        ),
    ])
}

pub fn hint_encountered_while_interpreting(
    context_description: &str,
    assignment_pos: FilePosition,
    error: &mut CompileProblem,
) {
    error.add_descriptor(ProblemDescriptor::new(
        assignment_pos,
        Hint,
        &format!(
            "Error encountered while interpreting {}:",
            context_description
        ),
    ));
}
