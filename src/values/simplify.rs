use std::collections::HashMap;

use itertools::Itertools;

use super::{LocalPtr, Operation, Value, ValuePtr};
use crate::types::{calculate_type_arithmetic, Type};

fn int_op(op: Operation, lhs: i32, rhs: i32) -> Value {
    match op {
        Operation::Add => Value::IntLiteral(lhs + rhs),
        Operation::Sub => Value::IntLiteral(lhs - rhs),
        Operation::Mul => Value::IntLiteral(lhs * rhs),
        Operation::Div => Value::IntLiteral(lhs / rhs),
        Operation::Rem => Value::IntLiteral(lhs % rhs),

        Operation::Gt => Value::BoolLiteral(lhs > rhs),
        Operation::Lt => Value::BoolLiteral(lhs < rhs),
        Operation::Gte => Value::BoolLiteral(lhs >= rhs),
        Operation::Lte => Value::BoolLiteral(lhs <= rhs),
        Operation::Eq => Value::BoolLiteral(lhs == rhs),
        Operation::Neq => Value::BoolLiteral(lhs != rhs),

        Operation::And => Value::IntLiteral(lhs & rhs),
        Operation::Or => Value::IntLiteral(lhs | rhs),
        Operation::Xor => Value::IntLiteral(lhs ^ rhs),

        Operation::Not => unreachable!(),
        Operation::Typeof => unreachable!(),
    }
}

fn float_op(op: Operation, lhs: f32, rhs: f32) -> Value {
    match op {
        Operation::Add => Value::FloatLiteral(lhs + rhs),
        Operation::Sub => Value::FloatLiteral(lhs - rhs),
        Operation::Mul => Value::FloatLiteral(lhs * rhs),
        Operation::Div => Value::FloatLiteral(lhs / rhs),
        Operation::Rem => Value::FloatLiteral(lhs % rhs),

        Operation::Gt => Value::BoolLiteral(lhs > rhs),
        Operation::Lt => Value::BoolLiteral(lhs < rhs),
        Operation::Gte => Value::BoolLiteral(lhs >= rhs),
        Operation::Lte => Value::BoolLiteral(lhs <= rhs),
        Operation::Eq => Value::BoolLiteral(lhs == rhs),
        Operation::Neq => Value::BoolLiteral(lhs != rhs),

        Operation::And => unreachable!(),
        Operation::Or => unreachable!(),
        Operation::Xor => unreachable!(),

        Operation::Not => unreachable!(),
        Operation::Typeof => unreachable!(),
    }
}

fn bool_op(op: Operation, lhs: bool, rhs: bool) -> Value {
    match op {
        Operation::Add => unreachable!(),
        Operation::Sub => unreachable!(),
        Operation::Mul => unreachable!(),
        Operation::Div => unreachable!(),
        Operation::Rem => unreachable!(),

        Operation::Gt => unreachable!(),
        Operation::Lt => unreachable!(),
        Operation::Gte => unreachable!(),
        Operation::Lte => unreachable!(),
        Operation::Eq => unreachable!(),
        Operation::Neq => unreachable!(),

        Operation::And => Value::BoolLiteral(lhs & rhs),
        Operation::Or => Value::BoolLiteral(lhs | rhs),
        Operation::Xor => Value::BoolLiteral(lhs ^ rhs),

        Operation::Not => unreachable!(),
        Operation::Typeof => unreachable!(),
    }
}

#[derive(Clone, Debug)]
pub struct SimplificationContext {
    locals: HashMap<LocalPtr, ValuePtr>,
}

impl SimplificationContext {
    pub fn new() -> Self {
        Self {
            locals: HashMap::new(),
        }
    }

    pub fn assignments(self) -> Vec<(LocalPtr, ValuePtr)> {
        self.locals.into_iter().collect()
    }
}

impl ValuePtr {
    pub fn typee(&self) -> Value {
        match &*self.0 {
            Value::TypeLiteral(..) => Value::TypeLiteral(Type::Type),
            Value::FloatLiteral(_) => Value::TypeLiteral(Type::Float),
            Value::IntLiteral(_) => Value::TypeLiteral(Type::Int),
            Value::BoolLiteral(_) => Value::TypeLiteral(Type::Bool),
            Value::Local(local) => (*local.typee).clone(),
            Value::Operation(op, args) => Value::Operation(
                *op,
                args.iter()
                    .map(ValuePtr::typee)
                    .map(ValuePtr::new)
                    .collect(),
            ),
            Value::Assignment { base, targets } => Value::TypeLiteral(Type::Malformed),
            Value::Function {
                inputs,
                outputs,
                locals,
                body,
            } => todo!(),
            Value::FunctionCall(_, _) => todo!(),
            Value::MultipleResults(_) => todo!(),
            Value::ExtractResult(base, index) => {
                Value::ExtractResult(ValuePtr::new(base.typee()), *index)
            }
            Value::Noop => Value::Noop,
            Value::Any => Value::Any,
        }
    }

    pub fn simplify(&self, ctx: &mut SimplificationContext) -> Self {
        match &*self.0 {
            Value::TypeLiteral(typ) => ValuePtr::new(Value::TypeLiteral(typ.simplify(ctx))),
            Value::FloatLiteral(_)
            | Value::IntLiteral(_)
            | Value::BoolLiteral(_)
            | Value::MultipleResults(_)
            | Value::Noop
            | Value::Any => self.ptr_clone(),
            Value::Function {
                inputs,
                outputs,
                locals,
                body,
            } => {
                let mut new_body = Vec::new();
                let mut new_ctx = ctx.clone();
                for statement in body {
                    new_body.push(statement.simplify(&mut new_ctx));
                }
                for (key, value) in &new_ctx.locals {
                    if !ctx.locals.contains_key(key) {
                        new_body.push(ValuePtr::new(Value::Assignment {
                            base: value.ptr_clone(),
                            targets: vec![key.ptr_clone()],
                        }));
                    }
                }
                ValuePtr::new(Value::Function {
                    inputs: inputs.clone(),
                    outputs: outputs.clone(),
                    locals: locals.clone(),
                    body: new_body,
                })
            }
            Value::Local(local) => {
                if let Some(value) = ctx.locals.get(local) {
                    value.ptr_clone()
                } else {
                    self.ptr_clone()
                }
            }
            Value::Operation(op, args) => {
                let args = args.iter().map(|x| x.simplify(ctx)).collect_vec();
                let new_value = if args.len() == 2 {
                    let mut args = args.clone().into_iter();
                    let lhs = args.next().unwrap();
                    let rhs = args.next().unwrap();
                    match (&*lhs, &*rhs) {
                        (&Value::IntLiteral(lhs), &Value::IntLiteral(rhs)) => {
                            Some(int_op(*op, lhs, rhs))
                        }
                        (&Value::FloatLiteral(lhs), &Value::IntLiteral(rhs)) => {
                            Some(float_op(*op, lhs, rhs as f32))
                        }
                        (&Value::IntLiteral(lhs), &Value::FloatLiteral(rhs)) => {
                            Some(float_op(*op, lhs as f32, rhs))
                        }
                        (&Value::FloatLiteral(lhs), &Value::FloatLiteral(rhs)) => {
                            Some(float_op(*op, lhs, rhs))
                        }
                        (&Value::BoolLiteral(lhs), &Value::BoolLiteral(rhs)) => {
                            Some(bool_op(*op, lhs, rhs))
                        }
                        (Value::TypeLiteral(lhs), Value::TypeLiteral(rhs)) => {
                            Some(calculate_type_arithmetic(*op, &[lhs.clone(), rhs.clone()]))
                        }
                        _ => None,
                    }
                } else if op == &Operation::Typeof {
                    let mut args = args.clone().into_iter();
                    let base = args.next().unwrap();
                    Some(base.typee())
                } else {
                    None
                };
                ValuePtr::new(new_value.unwrap_or_else(|| Value::Operation(*op, args)))
            }
            Value::Assignment { base, targets } => {
                let base = base.simplify(ctx);
                for (index, target) in targets.iter().enumerate() {
                    let value = if targets.len() == 1 {
                        base.ptr_clone()
                    } else {
                        ValuePtr::new(Value::ExtractResult(base.ptr_clone(), index))
                    };
                    ctx.locals.insert(target.ptr_clone(), value);
                }
                ValuePtr::new(Value::Noop)
            }
            Value::ExtractResult(base, index) => {
                let base = base.simplify(ctx);
                if let Value::MultipleResults(results) = &*base {
                    results[*index].ptr_clone()
                } else {
                    ValuePtr::new(Value::ExtractResult(base, *index))
                }
            }
            Value::FunctionCall(base, args) => {
                let base = base.simplify(ctx);
                let args = args.iter().map(|x| x.simplify(ctx)).collect_vec();
                if let Value::Function {
                    inputs,
                    outputs,
                    body,
                    ..
                } = &*base
                {
                    let mut new_ctx = ctx.clone();
                    assert_eq!(args.len(), inputs.len());
                    for (input, arg) in inputs.iter().zip(args.iter()) {
                        new_ctx.locals.insert(input.ptr_clone(), arg.ptr_clone());
                    }
                    for statement in body {
                        statement.simplify(&mut new_ctx);
                    }
                    let mut results = Vec::new();
                    for output in outputs {
                        if let Some(result) = new_ctx.locals.get(output) {
                            results.push(result.ptr_clone());
                        } else {
                            panic!("Output not assigned in function body.");
                        }
                    }
                    if results.len() == 1 {
                        results.into_iter().next().unwrap()
                    } else {
                        ValuePtr::new(Value::MultipleResults(results))
                    }
                } else {
                    ValuePtr::new(Value::FunctionCall(base, args))
                }
            }
        }
    }
}
