mod interpreter;
mod resolver;
pub(self) mod util;

use crate::problem::{CompileProblem, FilePosition};
use crate::structure::{Expression, KnownData, Program};

pub enum InterpreterOutcome {
    Specific(KnownData),
    Returned,
    Unknown,
    Problem(CompileProblem),
}

pub use interpreter::*;
pub use resolver::*;
