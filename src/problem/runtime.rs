use super::objects::*;
use ProblemType::Error;
use ProblemType::Hint;

pub fn assert_failed(pos: FilePosition) -> CompileProblem {
    CompileProblem::from_descriptors(vec![ProblemDescriptor::new(pos, Error, "Assert Failed")])
}
