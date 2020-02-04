mod ingest;
mod structure;
pub use structure::Program;

use crate::native::traits;
use crate::specialized::structure as i;

impl traits::Program for Program {
    fn new(input: &i::Program) -> Program {
        ingest::ingest(input)
    }

    unsafe fn execute(&self) -> i64 {
        Program::execute(self)
    }
}
