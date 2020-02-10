use crate::specialized::structure as i;

pub trait Program {
    fn new(input: &i::Program) -> Self;
    unsafe fn execute(&self) -> i64;
    unsafe fn set_input_i32(&mut self, index: usize, value: i32);
    // unsafe fn set_input_f32(&mut self, index: usize, value: i32);
    // unsafe fn set_input_bool(&mut self, index: usize, value: bool);
    unsafe fn read_output_i32(&self, index: usize) -> i32;
    // unsafe fn read_output_f32(&self, index: usize) -> f32;
    // unsafe fn read_output_bool(&self, index: usize) -> bool;
}
