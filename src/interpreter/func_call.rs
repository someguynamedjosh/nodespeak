use crate::problem::FilePosition;
use crate::structure::{BuiltinFunction, FuncCall, FunctionData, KnownData, Program, ScopeId};

pub enum InterpreterOutcome {
    Successful,
    Returned,
    UnknownData,
    UnknownFunction,
    AssertFailed(FilePosition),
}

fn interpret_builtin(
    program: &mut Program,
    scope: ScopeId,
    builtin: BuiltinFunction,
    func_call: &FuncCall,
) -> InterpreterOutcome {
    // All the inputs and outputs should have data types that match the function signatures for the
    // provided builtin.
    match builtin {
        // TODO: Handle array types.
        BuiltinFunction::Add => {
            let a = &func_call.borrow_inputs()[0];
            let b = &func_call.borrow_inputs()[1];
            let x = &func_call.borrow_outputs()[0];
            let x_data = match program.borrow_value_of(a) {
                KnownData::Bool(_value) => unimplemented!(),
                KnownData::Int(value) => {
                    KnownData::Int(value + program.borrow_value_of(b).require_int())
                }
                KnownData::Float(value) => {
                    KnownData::Float(value + program.borrow_value_of(b).require_float())
                }
                _ => unreachable!(),
            };
            program.set_value_of(x, x_data);
        }
        BuiltinFunction::Subtract => {
            let a = &func_call.borrow_inputs()[0];
            let b = &func_call.borrow_inputs()[1];
            let x = &func_call.borrow_outputs()[0];
            let x_data = match program.borrow_value_of(a) {
                KnownData::Bool(_value) => unimplemented!(),
                KnownData::Int(value) => {
                    KnownData::Int(value - program.borrow_value_of(b).require_int())
                }
                KnownData::Float(value) => {
                    KnownData::Float(value - program.borrow_value_of(b).require_float())
                }
                _ => unreachable!(),
            };
            program.set_value_of(x, x_data);
        }
        BuiltinFunction::Multiply => {
            let a = &func_call.borrow_inputs()[0];
            let b = &func_call.borrow_inputs()[1];
            let x = &func_call.borrow_outputs()[0];
            let x_data = match program.borrow_value_of(a) {
                KnownData::Bool(_value) => unimplemented!(),
                KnownData::Int(value) => {
                    KnownData::Int(value * program.borrow_value_of(b).require_int())
                }
                KnownData::Float(value) => {
                    KnownData::Float(value * program.borrow_value_of(b).require_float())
                }
                _ => unreachable!(),
            };
            program.set_value_of(x, x_data);
        }
        BuiltinFunction::Divide => {
            let a = &func_call.borrow_inputs()[0];
            let b = &func_call.borrow_inputs()[1];
            let x = &func_call.borrow_outputs()[0];
            let x_data = match program.borrow_value_of(a) {
                KnownData::Float(value) => {
                    KnownData::Float(value / program.borrow_value_of(b).require_float())
                }
                _ => unreachable!(),
            };
            program.set_value_of(x, x_data);
        }
        BuiltinFunction::IntDiv => {
            let a = &func_call.borrow_inputs()[0];
            let b = &func_call.borrow_inputs()[1];
            let x = &func_call.borrow_outputs()[0];
            let x_data = match program.borrow_value_of(a) {
                KnownData::Int(value) => {
                    KnownData::Int(value / program.borrow_value_of(b).require_int())
                }
                _ => unreachable!(),
            };
            program.set_value_of(x, x_data);
        }
        BuiltinFunction::Modulo => {
            let a = &func_call.borrow_inputs()[0];
            let b = &func_call.borrow_inputs()[1];
            let x = &func_call.borrow_outputs()[0];
            let x_data = match program.borrow_value_of(a) {
                KnownData::Bool(_value) => unimplemented!(),
                KnownData::Int(value) => {
                    KnownData::Int(value % program.borrow_value_of(b).require_int())
                }
                KnownData::Float(value) => {
                    KnownData::Float(value % program.borrow_value_of(b).require_float())
                }
                _ => unreachable!(),
            };
            program.set_value_of(x, x_data);
        }
        BuiltinFunction::Equal => {
            let a = &func_call.borrow_inputs()[0];
            let b = &func_call.borrow_inputs()[1];
            let x = &func_call.borrow_outputs()[0];
            let x_data = KnownData::Bool(program.borrow_value_of(a) == program.borrow_value_of(b));
            program.set_value_of(x, x_data);
        }
        BuiltinFunction::Copy => {
            let a = &func_call.borrow_inputs()[0];
            let x = &func_call.borrow_outputs()[0];
            let x_data = program.borrow_value_of(a).clone();
            program.set_value_of(x, x_data);
        }
        BuiltinFunction::Assert => {
            let a = &func_call.borrow_inputs()[0];
            match program.borrow_value_of(a) {
                KnownData::Bool(value) => {
                    if !value {
                        return InterpreterOutcome::AssertFailed(func_call.get_position().clone());
                    }
                }
                _ => unreachable!(),
            }
        }
        BuiltinFunction::Return => {
            return InterpreterOutcome::Returned;
        }
        _ => {
            eprintln!("{:?}", builtin);
            unimplemented!()
        }
    }
    InterpreterOutcome::Successful
}

fn interpret_function(
    program: &mut Program,
    func_call: &FuncCall,
    function: FunctionData,
) -> InterpreterOutcome {
    let input_sources = func_call.borrow_inputs();
    let input_targets = program
        .borrow_scope(function.get_body())
        .borrow_inputs()
        .clone();
    for (source, target) in input_sources.iter().zip(input_targets.iter()) {
        let value = program.borrow_value_of(source).clone();
        program.set_temporary_value(target.clone(), value);
    }
    let body = function.get_body();
    for func_call in program.clone_scope_body(body) {
        match interpret_func_call(program, body, &func_call) {
            InterpreterOutcome::Returned => break,
            _ => (),
        }
    }
    let output_sources = program
        .borrow_scope(function.get_body())
        .borrow_outputs()
        .clone();
    let output_targets = func_call.borrow_outputs();
    for (source, target) in output_sources.iter().zip(output_targets.iter()) {
        let value = program.borrow_temporary_value(source.clone()).clone();
        program.set_value_of(target, value);
    }
    InterpreterOutcome::Successful
}

/// Interprets the given function call. If its result can be determined, then the temporary values
/// of the func call's outputs are set to the values they would have if the function call was
/// actually executed, and the function returns true. If not, nothing happens and the function
/// returns false.
pub fn interpret_func_call(
    program: &mut Program,
    scope: ScopeId,
    func_call: &FuncCall,
) -> InterpreterOutcome {
    for input in func_call.borrow_inputs().iter() {
        // TODO: Handle array types, etc.
        match program.borrow_temporary_value(input.get_base()) {
            KnownData::Void | KnownData::Unknown => return InterpreterOutcome::UnknownData,
            _ => (),
        }
    }
    let function = func_call.get_function();
    match program.borrow_temporary_value(function) {
        KnownData::Function(function_data) => match function_data.get_builtin() {
            Option::Some(builtin) => {
                let cloned_builtin = builtin.clone();
                return interpret_builtin(program, scope, cloned_builtin, func_call);
            }
            Option::None => {
                let cloned_data = function_data.clone();
                return interpret_function(program, func_call, cloned_data);
            }
        },
        _ => return InterpreterOutcome::UnknownFunction,
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
    for func_call in program.clone_scope_body(program.get_entry_point()) {
        match interpret_func_call(program, entry_point_id, &func_call) {
            InterpreterOutcome::Returned => break,
            InterpreterOutcome::AssertFailed(_position) => {
                // TODO: Better error.
                return Result::Err("Assert failed.".to_owned())
            }
            InterpreterOutcome::UnknownData => {
                // TODO: Better error.
                return Result::Err("Inputs to function have unknown data.".to_owned())
            }
            InterpreterOutcome::UnknownFunction => {
                // TODO: Better error.
                return Result::Err("Could not determine which function to call.".to_owned())
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
