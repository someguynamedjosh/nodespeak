use crate::specialized::structure as i;

pub trait Program {
    fn new(input: &i::Program) -> Self;
    unsafe fn execute(&self) -> i64;
}
