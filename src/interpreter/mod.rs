mod interpreter;
mod resolver;
pub(self) mod util;

use crate::problem::{CompileProblem, FilePosition};
use crate::structure::{Expression, KnownData, Program};

pub enum InterpreterOutcome {
    Specific(KnownData),
    Returned,
    UnknownData,
    UnknownFunction,
    AssertFailed(FilePosition),
    Problem(CompileProblem),
}

pub use interpreter::*;
pub use resolver::*;
