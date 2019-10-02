use super::util;
use crate::interpreter::InterpreterOutcome;
use crate::problem::{self, CompileProblem, FilePosition};
use crate::structure::{Expression, KnownData, Program};
use std::borrow::Borrow;

struct Interpreter<'a> {
    program: &'a mut Program,
    error_on_unknown: bool,
}

use InterpreterOutcome::{Problem, Returned, Specific, Unknown};

fn wrap(
    context_description: &str,
    context_pos: FilePosition,
    mut base_problem: CompileProblem,
) -> InterpreterOutcome {
    problem::hint_encountered_while_interpreting(
        context_description,
        context_pos,
        &mut base_problem,
    );
    Problem(base_problem)
}

impl<'a> Interpreter<'a> {
    fn interpret(&mut self, expression: &Expression) -> InterpreterOutcome {
        // All the inputs and outputs should have data types that match the function signatures for the
        // provided builtin.
        match expression {
            Expression::Literal(data, ..) => Specific(data.clone()),
            Expression::Variable(id, ..) => {
                let data = self.program[*id].borrow_temporary_value().clone();
                match data {
                    KnownData::Unknown | KnownData::Void => {
                        if self.error_on_unknown {
                            Problem(problem::unknown_data(expression.clone_position()))
                        } else {
                            Unknown
                        }
                    }
                    _ => Specific(data),
                }
            }
            Expression::Access {
                base,
                indexes,
                position,
            } => {
                let base_value = match self.interpret(base) {
                    Specific(data) => match data {
                        KnownData::Array(data) => data,
                        KnownData::Unknown => return Unknown,
                        _ => {
                            if indexes.len() == 0 {
                                return Specific(data.clone());
                            } else {
                                return Problem(problem::indexing_non_array(
                                    base.clone_position(),
                                    position.clone(),
                                ));
                            }
                        }
                    },
                    Problem(problem) => return wrap("array index", position.clone(), problem),
                    Unknown => return Unknown,
                    _ => unreachable!(),
                };
                let mut index_values = Vec::with_capacity(indexes.len());
                for index in indexes {
                    index_values.push(match self.interpret(index) {
                        Specific(data) => match data {
                            // TODO: check that value is positive
                            KnownData::Int(value, ..) => value as usize,
                            _ => return Problem(problem::index_not_int(index.clone_position())),
                        },
                        Unknown => return Unknown,
                        Problem(problem) => return wrap("array index", position.clone(), problem),
                        _ => unreachable!(),
                    });
                }
                if index_values.len() == base_value.borrow_dimensions().len() {
                    if !base_value.is_inside(&index_values) {
                        return Problem(problem::index_not_in_range(
                            base.clone_position(),
                            position.clone(),
                            &index_values,
                        ));
                    }
                    Specific(base_value.borrow_item(&index_values).clone())
                } else {
                    if !base_value.is_slice_inside(&index_values) {
                        // TODO: Better error for when there are too many index operations.
                        return Problem(problem::index_not_in_range(
                            base.clone_position(),
                            position.clone(),
                            &index_values,
                        ));
                    }
                    Specific(KnownData::Array(base_value.clone_slice(&index_values)))
                }
            }
            Expression::BinaryOperation(a, operator, b, position) => {
                let a_result = self.interpret(a.borrow());
                let a_data = match a_result {
                    Specific(data, ..) => data,
                    Problem(problem) => return wrap("expression", position.clone(), problem),
                    _ => return a_result,
                };
                let b_result = self.interpret(b.borrow());
                let b_data = match b_result {
                    Specific(data, ..) => data,
                    Problem(problem) => return wrap("expression", position.clone(), problem),
                    _ => return b_result,
                };
                let result = util::compute_binary_operation(&a_data, *operator, &b_data);
                match result {
                    KnownData::Void | KnownData::Unknown => Unknown,
                    _ => Specific(result),
                }
            }
            Expression::Assign {
                target,
                value,
                position,
            } => {
                match self.interpret(value) {
                    Specific(result, ..) => self.program.set_value_of(target, result.clone()),
                    Problem(problem) => return wrap("assignment", position.clone(), problem),
                    Unknown => return Unknown,
                    _ => unreachable!(),
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
                    KnownData::Function(data) => data,

                    _ => return Unknown,
                };
                let body = func_data.get_body();
                let input_targets = self.program[body].borrow_inputs().clone();
                for (source, target) in inputs.iter().zip(input_targets.iter()) {
                    let value = self.program.borrow_value_of(source).clone();
                    if let KnownData::Unknown | KnownData::Void = value {
                        return Unknown;
                    }
                    self.program[*target].set_temporary_value(value);
                }
                for expression in self.program[body].borrow_body().clone() {
                    match self.interpret(&expression) {
                        Returned => break,
                        _ => (),
                    }
                }
                let output_sources = self.program[body].borrow_outputs().clone();
                let mut final_value = Option::None;
                for (source, target) in output_sources.iter().zip(outputs.iter()) {
                    let value = self.program[*source].borrow_temporary_value().clone();
                    match target {
                        // TODO error for multiple inline returns.
                        Expression::InlineReturn(..) => final_value = Option::Some(value),
                        _ => self.program.set_value_of(target, value),
                    }
                }
                Specific(final_value.unwrap_or(KnownData::Void))
            }
            Expression::Collect(expressions, position) => {
                let mut values = Vec::with_capacity(expressions.len());
                for expression in expressions.iter() {
                    match self.interpret(expression) {
                        Specific(data) => values.push(data),
                        Problem(problem) => {
                            return wrap("array literal", position.clone(), problem)
                        }
                        Unknown => return Unknown,
                        _ => unreachable!(),
                    }
                }
                if values.len() == 0 {
                    Specific(KnownData::new_array(vec![]))
                } else {
                    Specific(KnownData::collect(values))
                }
            }
            Expression::Assert(expression, assert_pos) => {
                let result = self.interpret(expression);
                match result {
                    Specific(value) => match value {
                        KnownData::Bool(bool_value) => {
                            if bool_value {
                                Specific(KnownData::Void)
                            } else {
                                Problem(problem::assert_failed(assert_pos.clone()))
                            }
                        }
                        _ => {
                            return Problem(problem::assert_not_bool(
                                expression.clone_position(),
                                assert_pos.clone(),
                            ))
                        }
                    },
                    Problem(problem) => return wrap("assertion", assert_pos.clone(), problem),
                    Unknown => return Unknown,
                    _ => unreachable!(),
                }
            }
            _ => {
                eprintln!("{:?}", expression);
                unimplemented!()
            }
        }
    }

    fn from_entry_point(
        &mut self,
        inputs: Vec<KnownData>,
    ) -> Result<Vec<KnownData>, CompileProblem> {
        let entry_point_id = self.program.get_entry_point();
        let entry_point = &self.program[entry_point_id];

        let input_iter = inputs.iter();
        let targets_iter = entry_point.borrow_inputs().iter();
        for (index, (input, target)) in input_iter.zip(targets_iter).enumerate() {
            if !input.matches_data_type(self.program[*target].borrow_data_type()) {
                panic!("Input at index {} has incorrect type.", index);
            }
        }

        let input_targets = entry_point.borrow_inputs().clone();
        for (source, target) in inputs.into_iter().zip(input_targets.iter()) {
            self.program[*target].set_temporary_value(source);
        }
        for expression in self.program[self.program.get_entry_point()]
            .borrow_body()
            .clone()
        {
            match self.interpret(&expression) {
                Returned => break,
                Problem(problem) => return Result::Err(problem),
                _ => (),
            }
        }

        let output_sources = self.program[entry_point_id].borrow_outputs().clone();
        let mut output_values = Vec::new();
        for source in output_sources.iter() {
            let value = self.program[*source].borrow_temporary_value().clone();
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
) -> Result<Vec<KnownData>, CompileProblem> {
    Interpreter {
        program,
        error_on_unknown: true,
    }
    .from_entry_point(inputs)
}
