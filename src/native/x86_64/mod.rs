mod storage;
pub use storage::Program;

use crate::native::traits;
use crate::trivial::structure as i;

impl traits::Program for Program {
    fn new(input: &i::Program) -> Program {
        unimplemented!()
    }

    unsafe fn execute(&self) -> i64 {
        storage::Program::execute(self)
    }
}
