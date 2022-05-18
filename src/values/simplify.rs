use std::collections::HashMap;

use itertools::Itertools;

use super::{BuiltinOp, BuiltinType, LocalPtr, Value, ValuePtr};

fn int_op(op: BuiltinOp, lhs: i32, rhs: i32) -> Value {
    match op {
        BuiltinOp::Add => Value::IntLiteral(lhs + rhs),
        BuiltinOp::Sub => Value::IntLiteral(lhs - rhs),
        BuiltinOp::Mul => Value::IntLiteral(lhs * rhs),
        BuiltinOp::Div => Value::IntLiteral(lhs / rhs),
        BuiltinOp::Rem => Value::IntLiteral(lhs % rhs),

        BuiltinOp::Gt => Value::BoolLiteral(lhs > rhs),
        BuiltinOp::Lt => Value::BoolLiteral(lhs < rhs),
        BuiltinOp::Gte => Value::BoolLiteral(lhs >= rhs),
        BuiltinOp::Lte => Value::BoolLiteral(lhs <= rhs),
        BuiltinOp::Eq => Value::BoolLiteral(lhs == rhs),
        BuiltinOp::Neq => Value::BoolLiteral(lhs != rhs),

        BuiltinOp::And => Value::IntLiteral(lhs & rhs),
        BuiltinOp::Or => Value::IntLiteral(lhs | rhs),
        BuiltinOp::Xor => Value::IntLiteral(lhs ^ rhs),

        BuiltinOp::Not => unreachable!(),
        BuiltinOp::Typeof => unreachable!(),
    }
}

fn float_op(op: BuiltinOp, lhs: f32, rhs: f32) -> Value {
    match op {
        BuiltinOp::Add => Value::FloatLiteral(lhs + rhs),
        BuiltinOp::Sub => Value::FloatLiteral(lhs - rhs),
        BuiltinOp::Mul => Value::FloatLiteral(lhs * rhs),
        BuiltinOp::Div => Value::FloatLiteral(lhs / rhs),
        BuiltinOp::Rem => Value::FloatLiteral(lhs % rhs),

        BuiltinOp::Gt => Value::BoolLiteral(lhs > rhs),
        BuiltinOp::Lt => Value::BoolLiteral(lhs < rhs),
        BuiltinOp::Gte => Value::BoolLiteral(lhs >= rhs),
        BuiltinOp::Lte => Value::BoolLiteral(lhs <= rhs),
        BuiltinOp::Eq => Value::BoolLiteral(lhs == rhs),
        BuiltinOp::Neq => Value::BoolLiteral(lhs != rhs),

        BuiltinOp::And => unreachable!(),
        BuiltinOp::Or => unreachable!(),
        BuiltinOp::Xor => unreachable!(),

        BuiltinOp::Not => unreachable!(),
        BuiltinOp::Typeof => unreachable!(),
    }
}

fn bool_op(op: BuiltinOp, lhs: bool, rhs: bool) -> Value {
    match op {
        BuiltinOp::Add => unreachable!(),
        BuiltinOp::Sub => unreachable!(),
        BuiltinOp::Mul => unreachable!(),
        BuiltinOp::Div => unreachable!(),
        BuiltinOp::Rem => unreachable!(),

        BuiltinOp::Gt => unreachable!(),
        BuiltinOp::Lt => unreachable!(),
        BuiltinOp::Gte => unreachable!(),
        BuiltinOp::Lte => unreachable!(),
        BuiltinOp::Eq => unreachable!(),
        BuiltinOp::Neq => unreachable!(),

        BuiltinOp::And => Value::BoolLiteral(lhs & rhs),
        BuiltinOp::Or => Value::BoolLiteral(lhs | rhs),
        BuiltinOp::Xor => Value::BoolLiteral(lhs ^ rhs),

        BuiltinOp::Not => unreachable!(),
        BuiltinOp::Typeof => unreachable!(),
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
            Value::BuiltinType(typ) => match typ {
                BuiltinType::Int
                | BuiltinType::Float
                | BuiltinType::Bool
                | BuiltinType::Type
                | BuiltinType::Array { .. }
                | BuiltinType::InSet { .. }
                | BuiltinType::Function { .. }
                | BuiltinType::Malformed => Value::BuiltinType(BuiltinType::Type),
            },
            Value::Noop | Value::Any => Value::Any,
            Value::Malformed => Value::BuiltinType(BuiltinType::Malformed),
            Value::BuiltinOp(op) => match op {
                BuiltinOp::Add
                | BuiltinOp::Sub
                | BuiltinOp::Mul
                | BuiltinOp::Div
                | BuiltinOp::Rem
                | BuiltinOp::Gt
                | BuiltinOp::Lt
                | BuiltinOp::Gte
                | BuiltinOp::Lte
                | BuiltinOp::Eq
                | BuiltinOp::Neq
                | BuiltinOp::And
                | BuiltinOp::Or
                | BuiltinOp::Xor => todo!(),
                BuiltinOp::Not | BuiltinOp::Typeof => todo!(),
            },
            Value::ArrayLiteral {
                elements: _,
                dims: _,
            } => todo!(),
            Value::FloatLiteral(_) => Value::BuiltinType(BuiltinType::Float),
            Value::IntLiteral(_) => Value::BuiltinType(BuiltinType::Int),
            Value::BoolLiteral(_) => Value::BuiltinType(BuiltinType::Bool),
            Value::Local(local) => (*local.typee).clone(),
            Value::Assignment { .. } => Value::BuiltinType(BuiltinType::Malformed),
            Value::Function {
                inputs: _,
                outputs: _,
                locals: _,
                body: _,
            } => todo!(),
            Value::FunctionCall(_, _, _) => todo!(),
        }
    }

    pub fn simplify(&self, ctx: &mut SimplificationContext) -> Self {
        match &*self.0 {
            Value::BuiltinType(BuiltinType::Array { eltype, dims }) => {
                Self::new(Value::BuiltinType(BuiltinType::Array {
                    eltype: eltype.simplify(ctx),
                    dims: dims.iter().map(|x| x.simplify(ctx)).collect(),
                }))
            }
            Value::BuiltinType(BuiltinType::InSet { eltype, elements }) => {
                Self::new(Value::BuiltinType(BuiltinType::InSet {
                    eltype: eltype.simplify(ctx),
                    elements: elements.iter().map(|x| x.simplify(ctx)).collect(),
                }))
            }
            Value::BuiltinType(BuiltinType::Function { inputs, outputs }) => {
                Self::new(Value::BuiltinType(BuiltinType::Function {
                    inputs: inputs.iter().map(|x| x.simplify(ctx)).collect(),
                    outputs: outputs.iter().map(|x| x.simplify(ctx)).collect(),
                }))
            }
            Value::FloatLiteral(_)
            | Value::IntLiteral(_)
            | Value::BoolLiteral(_)
            | Value::BuiltinType(_)
            | Value::BuiltinOp(_)
            | Value::Any
            | Value::Malformed
            | Value::Noop => self.ptr_clone(),
            Value::ArrayLiteral { elements, dims } => Self::new(Value::ArrayLiteral {
                elements: elements.iter().map(|x| x.simplify(ctx)).collect(),
                dims: dims.iter().map(|x| x.simplify(ctx)).collect(),
            }),
            Value::Assignment { base, target } => {
                let base = base.simplify(ctx);
                ctx.locals.insert(target.ptr_clone(), base.ptr_clone());
                ValuePtr::new(Value::Noop)
            }
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
                            target: key.ptr_clone(),
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
            Value::FunctionCall(base, args, output) => {
                let base = base.simplify(ctx);
                let args = args.iter().map(|x| x.simplify(ctx)).collect_vec();
                if let Value::BuiltinOp(builtin) = &*base {
                    let new_value = if args.len() == 2 {
                        let mut args = args.clone().into_iter();
                        let lhs = args.next().unwrap();
                        let rhs = args.next().unwrap();
                        match (&*lhs, &*rhs) {
                            (&Value::IntLiteral(lhs), &Value::IntLiteral(rhs)) => {
                                Some(int_op(*builtin, lhs, rhs))
                            }
                            (&Value::FloatLiteral(lhs), &Value::IntLiteral(rhs)) => {
                                Some(float_op(*builtin, lhs, rhs as f32))
                            }
                            (&Value::IntLiteral(lhs), &Value::FloatLiteral(rhs)) => {
                                Some(float_op(*builtin, lhs as f32, rhs))
                            }
                            (&Value::FloatLiteral(lhs), &Value::FloatLiteral(rhs)) => {
                                Some(float_op(*builtin, lhs, rhs))
                            }
                            (&Value::BoolLiteral(lhs), &Value::BoolLiteral(rhs)) => {
                                Some(bool_op(*builtin, lhs, rhs))
                            }
                            // (Value::Builtin(lhs), Value::TypeLiteral(rhs)) => {
                            //     Some(calculate_type_arithmetic(*builtin, &[lhs.clone(),
                            // rhs.clone()])) }
                            _ => None,
                        }
                    } else if builtin == &BuiltinOp::Typeof {
                        let mut args = args.clone().into_iter();
                        let base = args.next().unwrap();
                        Some(base.typee())
                    } else {
                        None
                    };
                    if let Some(new_value) = new_value {
                        return ValuePtr::new(new_value);
                    }
                } else if let Value::Function {
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
                    return results.into_iter().skip(*output).next().unwrap();
                }
                ValuePtr::new(Value::FunctionCall(base, args, *output))
            }
            Value::Local(local) => {
                if let Some(value) = ctx.locals.get(local) {
                    value.ptr_clone()
                } else {
                    self.ptr_clone()
                }
            }
        }
    }
}
