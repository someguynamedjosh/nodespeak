use crate::trivial::structure as i;

pub trait Program {
    fn new(input: &i::Program) -> Self;
    unsafe fn execute(&self) -> i64;
}
