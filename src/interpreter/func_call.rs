use crate::problem::FilePosition;
use crate::structure::{Expression, KnownData, Program};

pub enum InterpreterOutcome {
    Successful(KnownData),
    Returned,
    UnknownData,
    UnknownFunction,
    AssertFailed(FilePosition),
}

pub fn interpret_expression(program: &mut Program, expression: &Expression) -> InterpreterOutcome {
    // All the inputs and outputs should have data types that match the function signatures for the
    // provided builtin.
    match expression {
        // TODO: Handle array types.
        Expression::Add(a, b) => InterpreterOutcome::Successful(match program.borrow_value_of(a) {
            KnownData::Bool(_value) => unimplemented!(),
            KnownData::Int(value) => {
                KnownData::Int(value + program.borrow_value_of(b).require_int())
            }
            KnownData::Float(value) => {
                KnownData::Float(value + program.borrow_value_of(b).require_float())
            }
            _ => unreachable!(),
        }),
        Expression::Subtract(a, b) => {
            InterpreterOutcome::Successful(match program.borrow_value_of(a) {
                KnownData::Bool(_value) => unimplemented!(),
                KnownData::Int(value) => {
                    KnownData::Int(value - program.borrow_value_of(b).require_int())
                }
                KnownData::Float(value) => {
                    KnownData::Float(value - program.borrow_value_of(b).require_float())
                }
                _ => unreachable!(),
            })
        }
        Expression::Multiply(a, b) => {
            InterpreterOutcome::Successful(match program.borrow_value_of(a) {
                KnownData::Bool(_value) => unimplemented!(),
                KnownData::Int(value) => {
                    KnownData::Int(value * program.borrow_value_of(b).require_int())
                }
                KnownData::Float(value) => {
                    KnownData::Float(value * program.borrow_value_of(b).require_float())
                }
                _ => unreachable!(),
            })
        }
        Expression::Divide(a, b) => {
            InterpreterOutcome::Successful(match program.borrow_value_of(a) {
                KnownData::Float(value) => {
                    KnownData::Float(value / program.borrow_value_of(b).require_float())
                }
                _ => unreachable!(),
            })
        }
        Expression::IntDiv(a, b) => {
            InterpreterOutcome::Successful(match program.borrow_value_of(a) {
                KnownData::Int(value) => {
                    KnownData::Int(value / program.borrow_value_of(b).require_int())
                }
                _ => unreachable!(),
            })
        }
        Expression::Modulo(a, b) => {
            InterpreterOutcome::Successful(match program.borrow_value_of(a) {
                KnownData::Bool(_value) => unimplemented!(),
                KnownData::Int(value) => {
                    KnownData::Int(value % program.borrow_value_of(b).require_int())
                }
                KnownData::Float(value) => {
                    KnownData::Float(value % program.borrow_value_of(b).require_float())
                }
                _ => unreachable!(),
            })
        }
        Expression::Equal(a, b) => InterpreterOutcome::Successful(KnownData::Bool(
            program.borrow_value_of(a) == program.borrow_value_of(b),
        )),
        Expression::Assign { target, value } => {
            program.set_value_of(target, program.borrow_value_of(value).clone());
            InterpreterOutcome::Successful(KnownData::Void)
        }
        Expression::FuncCall {
            function,
            inputs,
            outputs,
        } => {
            let func_data = match program.borrow_value_of(function) {
                KnownData::Function(data) => data,
                _ => return InterpreterOutcome::UnknownFunction,
            };
            let body = func_data.get_body();
            let input_targets = program.borrow_scope(body).borrow_inputs().clone();
            for (source, target) in inputs.iter().zip(input_targets.iter()) {
                let value = program.borrow_value_of(source).clone();
                if let KnownData::Unknown | KnownData::Void = value {
                    return InterpreterOutcome::UnknownData;
                }
                program.set_temporary_value(target.clone(), value);
            }
            for expression in program.clone_scope_body(body) {
                match interpret_expression(program, &expression) {
                    InterpreterOutcome::Returned => break,
                    _ => (),
                }
            }
            let output_sources = program.borrow_scope(body).borrow_outputs().clone();
            let mut final_value = Option::None;
            for (source, target) in output_sources.iter().zip(outputs.iter()) {
                let value = program.borrow_temporary_value(*source).clone();
                match target {
                    // TODO error for multiple inline returns.
                    Expression::InlineReturn => final_value = Option::Some(value),
                    _ => program.set_value_of(target, value),
                }
            }
            InterpreterOutcome::Successful(final_value.unwrap_or(KnownData::Void))
        }
        _ => {
            eprintln!("{:?}", expression);
            unimplemented!()
        }
    }
}

/// Interprets a program starting from its entry point. Uses the provided vector of data to
/// initialize all the input variables. If any of the input data do not match the types of their
/// respective input variables, the function panics.
pub fn interpret_from_entry_point(
    program: &mut Program,
    inputs: Vec<KnownData>,
) -> Result<Vec<KnownData>, String> {
    let entry_point_id = program.get_entry_point();
    let entry_point = program.borrow_scope(entry_point_id);

    let input_iter = inputs.iter();
    let targets_iter = entry_point.borrow_inputs().iter();
    for (index, (input, target)) in input_iter.zip(targets_iter).enumerate() {
        if !input.matches_data_type(program.borrow_variable(*target).borrow_data_type()) {
            panic!("Input at index {} has incorrect type.", index);
        }
    }

    let input_targets = entry_point.borrow_inputs().clone();
    for (source, target) in inputs.into_iter().zip(input_targets.iter()) {
        program.set_temporary_value(target.clone(), source);
    }
    for expression in program.clone_scope_body(program.get_entry_point()) {
        match interpret_expression(program, &expression) {
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
                return Result::Err("Could not determine which function to expression.".to_owned());
            }
            _ => (),
        }
    }

    let output_sources = program
        .borrow_scope(entry_point_id)
        .borrow_outputs()
        .clone();
    let mut output_values = Vec::new();
    for source in output_sources.iter() {
        let value = program.borrow_temporary_value(source.clone()).clone();
        output_values.push(value);
    }
    Result::Ok(output_values)
}
