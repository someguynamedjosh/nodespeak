use crate::shared::NativeType;
use crate::specialized::structure as i;

pub trait Program {
    fn new(input: &i::Program) -> Self;
    unsafe fn execute(&self) -> i64;
    fn set_input_i32(&mut self, index: usize, value: i32);
    fn set_input_f32(&mut self, index: usize, value: f32);
    // fn set_input_bool(&mut self, index: usize, value: bool);
    fn read_output_i32(&self, index: usize) -> i32;
    fn read_output_f32(&self, index: usize) -> f32;
    // fn read_output_bool(&self, index: usize) -> bool;
    fn list_inputs(&self) -> Vec<NativeType>;
    fn list_outputs(&self) -> Vec<NativeType>;
}
