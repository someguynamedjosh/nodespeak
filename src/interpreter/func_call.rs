use crate::structure::{BuiltinFunction, FuncCall, KnownData, Program, ScopeId, FunctionData};
use crate::problem::FilePosition;

pub enum InterpreterOutcome {
    Successful,
    Returned,
    UnknownData,
    UnknownFunction,
    AssertFailed(FilePosition)
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

fn interpret_function(program: &mut Program, func_call: &FuncCall, function: FunctionData) -> InterpreterOutcome {
    let input_sources = func_call.borrow_inputs().iter();
    let input_targets = function.borrow_inputs().iter();
    for (source, target) in input_sources.zip(input_targets) {
        let value = program.borrow_value_of(source).clone();
        program.set_temporary_value(target.clone(), value);
    }
    let body = function.get_body();
    for func_call in program.clone_scope_body(body) {
        match interpret_func_call(program, body, &func_call) {
            InterpreterOutcome::Returned => break,
            _ => ()
        }
    }
    let output_sources = function.borrow_outputs().iter();
    let output_targets = func_call.borrow_outputs().iter();
    for (source, target) in output_sources.zip(output_targets) {
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
