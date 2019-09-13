use super::util;
use crate::interpreter::InterpreterOutcome;
use crate::problem::{CompileProblem, FilePosition};
use crate::structure::{Expression, KnownData, Program};
use std::borrow::Borrow;

struct Interpreter<'a> {
    program: &'a mut Program,
    error_on_unknown: bool,
}

use InterpreterOutcome::{Specific, UnknownData};

impl<'a> Interpreter<'a> {
    fn interpret(&mut self, expression: &Expression) -> InterpreterOutcome {
        // All the inputs and outputs should have data types that match the function signatures for the
        // provided builtin.
        match expression {
            Expression::Literal(data, ..) => Specific(data.clone()),
            Expression::Variable(id, ..) => {
                let data = self.program.borrow_temporary_value(*id).clone();
                match data {
                    KnownData::Unknown | KnownData::Void => UnknownData,
                    _ => Specific(data),
                }
            }
            Expression::Access { base, indexes, .. } => {
                let base_value = match self.interpret(base) {
                    Specific(data, ..) => match data {
                        KnownData::Array(data, ..) => data,
                        _ => {
                            if indexes.len() == 0 {
                                return Specific(data.clone());
                            } else {
                                panic!("TODO nice error, trying to index something that isn't an array.")
                            }
                        }
                    },
                    _ => {
                        panic!("TODO: nice error, could not interpret target of access expression.")
                    }
                };
                let mut index_values = Vec::with_capacity(indexes.len());
                for index in indexes {
                    index_values.push(match self.interpret(index) {
                        Specific(data, ..) => match data {
                            // TODO: check that value is positive
                            KnownData::Int(value, ..) => value as usize,
                            _ => panic!("TODO: nice error"),
                        },
                        _ => panic!("TODO: nice error"),
                    });
                }
                if index_values.len() == base_value.borrow_dimensions().len() {
                    if !base_value.is_inside(&index_values) {
                        panic!("TODO: nice error, index not in range.")
                    }
                    Specific(base_value.borrow_item(&index_values).clone())
                } else {
                    if !base_value.is_slice_inside(&index_values) {
                        panic!("TODO: nice error, index not in range.")
                    }
                    Specific(KnownData::Array(base_value.clone_slice(&index_values)))
                }
            }
            Expression::BinaryOperation(a, operator, b, ..) => {
                let a_result = self.interpret(a.borrow());
                let a_data = match a_result {
                    InterpreterOutcome::Specific(data, ..) => data,
                    _ => return a_result,
                };
                let b_result = self.interpret(b.borrow());
                let b_data = match b_result {
                    InterpreterOutcome::Specific(data, ..) => data,
                    _ => return b_result,
                };
                let result = util::compute_binary_operation(&a_data, *operator, &b_data);
                match result {
                    KnownData::Void | KnownData::Unknown => InterpreterOutcome::UnknownData,
                    _ => InterpreterOutcome::Specific(result),
                }
            }
            Expression::Assign { target, value, .. } => {
                match self.interpret(value) {
                    Specific(result, ..) => self.program.set_value_of(target, result.clone()),
                    _ => panic!("TODO: error, interpretation of {:?} failed.", value),
                }
                Specific(KnownData::Void)
            }
            Expression::FuncCall {
                function,
                inputs,
                outputs,
                ..
            } => {
                let func_data = match self.program.borrow_value_of(function) {
                    KnownData::Function(data, ..) => data,
                    _ => return InterpreterOutcome::UnknownFunction,
                };
                let body = func_data.get_body();
                let input_targets = self.program.borrow_scope(body).borrow_inputs().clone();
                for (source, target) in inputs.iter().zip(input_targets.iter()) {
                    let value = self.program.borrow_value_of(source).clone();
                    if let KnownData::Unknown | KnownData::Void = value {
                        return InterpreterOutcome::UnknownData;
                    }
                    self.program.set_temporary_value(target.clone(), value);
                }
                for expression in self.program.clone_scope_body(body) {
                    match self.interpret(&expression) {
                        InterpreterOutcome::Returned => break,
                        _ => (),
                    }
                }
                let output_sources = self.program.borrow_scope(body).borrow_outputs().clone();
                let mut final_value = Option::None;
                for (source, target) in output_sources.iter().zip(outputs.iter()) {
                    let value = self.program.borrow_temporary_value(*source).clone();
                    match target {
                        // TODO error for multiple inline returns.
                        Expression::InlineReturn(..) => final_value = Option::Some(value),
                        _ => self.program.set_value_of(target, value),
                    }
                }
                Specific(final_value.unwrap_or(KnownData::Void))
            }
            Expression::Collect(expressions, ..) => {
                let mut values = Vec::with_capacity(expressions.len());
                for expression in expressions.iter() {
                    match self.interpret(expression) {
                        Specific(data) => values.push(data),
                        _ => panic!("TODO: nice error, failed to interpret value."),
                    }
                }
                if values.len() == 0 {
                    Specific(KnownData::new_array(vec![]))
                } else {
                    Specific(KnownData::collect(values))
                }
            }
            Expression::Assert(expression, ..) => {
                let result = self.interpret(expression);
                if let Specific(value) = result {
                    match value {
                        KnownData::Bool(bool_value) => {
                            if bool_value {
                                Specific(KnownData::Void)
                            } else {
                                InterpreterOutcome::AssertFailed(FilePosition::placeholder())
                            }
                        }
                        _ => panic!("TODO: nice error, input to assert is not bool."),
                    }
                } else {
                    panic!("TODO: nice error, input to assert could not be interpreted.")
                }
            }
            _ => {
                eprintln!("{:?}", expression);
                unimplemented!()
            }
        }
    }

    fn from_entry_point(&mut self, inputs: Vec<KnownData>) -> Result<Vec<KnownData>, String> {
        let entry_point_id = self.program.get_entry_point();
        let entry_point = self.program.borrow_scope(entry_point_id);

        let input_iter = inputs.iter();
        let targets_iter = entry_point.borrow_inputs().iter();
        for (index, (input, target)) in input_iter.zip(targets_iter).enumerate() {
            if !input.matches_data_type(self.program.borrow_variable(*target).borrow_data_type()) {
                panic!("Input at index {} has incorrect type.", index);
            }
        }

        let input_targets = entry_point.borrow_inputs().clone();
        for (source, target) in inputs.into_iter().zip(input_targets.iter()) {
            self.program.set_temporary_value(target.clone(), source);
        }
        for expression in self
            .program
            .clone_scope_body(self.program.get_entry_point())
        {
            match self.interpret(&expression) {
                InterpreterOutcome::Returned => break,
                InterpreterOutcome::AssertFailed(_position) => {
                    // TODO: Better error.
                    return Result::Err("Assert failed.".to_owned());
                }
                InterpreterOutcome::UnknownData => {
                    // TODO: Better error.
                    return Result::Err("Inputs to function have unknown data.".to_owned());
                }
                InterpreterOutcome::UnknownFunction => {
                    // TODO: Better error.
                    return Result::Err(
                        "Could not determine which function to expression.".to_owned(),
                    );
                }
                _ => (),
            }
        }

        let output_sources = self
            .program
            .borrow_scope(entry_point_id)
            .borrow_outputs()
            .clone();
        let mut output_values = Vec::new();
        for source in output_sources.iter() {
            let value = self.program.borrow_temporary_value(source.clone()).clone();
            output_values.push(value);
        }
        Result::Ok(output_values)
    }
}

pub fn interpret_expression(
    program: &mut Program,
    expression: &Expression,
    error_on_unknown: bool,
) -> InterpreterOutcome {
    Interpreter {
        program,
        error_on_unknown,
    }
    .interpret(expression)
}

/// Interprets a self.program starting from its entry point. Uses the provided vector of data to
/// initialize all the input variables. If any of the input data do not match the types of their
/// respective input variables, the function panics.
pub fn interpret_from_entry_point(
    program: &mut Program,
    inputs: Vec<KnownData>,
) -> Result<Vec<KnownData>, String> {
    Interpreter {
        program,
        error_on_unknown: true,
    }
    .from_entry_point(inputs)
}
