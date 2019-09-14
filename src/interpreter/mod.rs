mod interpreter;
mod resolver;
pub(self) mod util;

use crate::problem::CompileProblem;
use crate::structure::KnownData;

pub enum InterpreterOutcome {
    Specific(KnownData),
    Returned,
    Unknown,
    Problem(CompileProblem),
}

pub use interpreter::*;
pub use resolver::*;
