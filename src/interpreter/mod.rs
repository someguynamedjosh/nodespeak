mod interpreter;
pub(self) mod util;

use crate::problem::{FilePosition, CompileProblem};
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
