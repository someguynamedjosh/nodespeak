use std::collections::HashMap;

use itertools::Itertools;

use super::{
    type_arithmetic::calculate_type_arithmetic, BuiltinOp, BuiltinType, LocalPtr, Value, ValuePtr,
};

fn int_op(op: BuiltinOp, lhs: i32, rhs: i32) -> Value {
    match op {
        BuiltinOp::Add => Value::IntLiteral(lhs + rhs),
        BuiltinOp::Sub => Value::IntLiteral(lhs - rhs),
        BuiltinOp::Mul => Value::IntLiteral(lhs * rhs),
        BuiltinOp::Div => Value::IntLiteral(lhs / rhs),
        BuiltinOp::Rem => Value::IntLiteral(lhs % rhs),

        BuiltinOp::Min => Value::IntLiteral(lhs.min(rhs)),
        BuiltinOp::Max => Value::IntLiteral(lhs.max(rhs)),

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

        BuiltinOp::Min => Value::FloatLiteral(lhs.min(rhs)),
        BuiltinOp::Max => Value::FloatLiteral(lhs.max(rhs)),

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

        BuiltinOp::Min => Value::BoolLiteral(lhs.min(rhs)),
        BuiltinOp::Max => Value::BoolLiteral(lhs.max(rhs)),

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

    pub fn finish(self) -> Vec<(LocalPtr, ValuePtr)> {
        self.locals.into_iter().collect()
    }
}

impl ValuePtr {
    pub fn typee(&self) -> Value {
        match &*self.0.borrow() {
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
                | BuiltinOp::Min
                | BuiltinOp::Max
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
            Value::Local(local) => (local.typee.borrow()).clone(),
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

    pub fn simplify(&self, ctx: &mut SimplificationContext) {
        let new_val = match &*self.0.borrow() {
            Value::BuiltinType(BuiltinType::Array { eltype, dims }) => {
                eltype.simplify(ctx);
                dims.iter().for_each(|x| x.simplify(ctx));
                None
            }
            Value::BuiltinType(BuiltinType::InSet { eltype, elements }) => {
                eltype.simplify(ctx);
                elements.iter().for_each(|x| x.simplify(ctx));
                None
            }
            Value::BuiltinType(BuiltinType::Function { inputs, outputs }) => {
                inputs.iter().for_each(|x| x.simplify(ctx));
                outputs.iter().for_each(|x| x.simplify(ctx));
                None
            }
            Value::FloatLiteral(_)
            | Value::IntLiteral(_)
            | Value::BoolLiteral(_)
            | Value::BuiltinType(_)
            | Value::BuiltinOp(_)
            | Value::Any
            | Value::Malformed
            | Value::Noop => None,
            Value::ArrayLiteral { elements, dims } => {
                elements.iter().for_each(|x| x.simplify(ctx));
                dims.iter().for_each(|x| x.simplify(ctx));
                None
            }
            Value::Assignment { base, target } => {
                base.simplify(ctx);
                ctx.locals.insert(target.ptr_clone(), base.ptr_clone());
                Some(Value::Noop)
            }
            Value::Function {
                inputs,
                outputs,
                locals,
                body,
            } => {
                let mut new_body = body.clone();
                let mut new_ctx = ctx.clone();
                for statement in body {
                    statement.simplify(&mut new_ctx);
                }
                for (key, value) in &new_ctx.locals {
                    if !ctx.locals.contains_key(key) {
                        new_body.push(ValuePtr::new(Value::Assignment {
                            base: value.ptr_clone(),
                            target: key.ptr_clone(),
                        }));
                    }
                }
                Some(Value::Function {
                    inputs: inputs.clone(),
                    outputs: outputs.clone(),
                    locals: locals.clone(),
                    body: new_body,
                })
            }
            Value::FunctionCall(base, args, output) => {
                base.simplify(ctx);
                args.iter().for_each(|x| x.simplify(ctx));
                if let Value::BuiltinOp(builtin) = &*base.borrow() {
                    let new_value = if args.len() == 2 {
                        let mut args = args.clone().into_iter();
                        let lhs = args.next().unwrap();
                        let rhs = args.next().unwrap();
                        let res = match (&*lhs.borrow(), &*rhs.borrow()) {
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
                            (Value::BuiltinType(lhs), Value::BuiltinType(rhs)) => Some(
                                calculate_type_arithmetic(*builtin, &[lhs.clone(), rhs.clone()]),
                            ),
                            _ => None,
                        };
                        res
                    } else if builtin == &BuiltinOp::Typeof {
                        let mut args = args.clone().into_iter();
                        let base = args.next().unwrap();
                        Some(base.typee())
                    } else {
                        None
                    };
                    new_value.or(Some(Value::FunctionCall(
                        base.ptr_clone(),
                        args.clone(),
                        *output,
                    )))
                } else if let Value::Function {
                    inputs,
                    outputs,
                    body,
                    ..
                } = &*base.borrow()
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
                    Some(
                        results
                            .into_iter()
                            .skip(*output)
                            .next()
                            .unwrap()
                            .borrow()
                            .clone(),
                    )
                } else {
                    Some(Value::FunctionCall(base.ptr_clone(), args.clone(), *output))
                }
            }
            Value::Local(local) => {
                if let Some(value) = ctx.locals.get(local) {
                    Some(value.borrow().clone())
                } else {
                    None
                }
            }
        };
        if let Some(new_val) = new_val {
            *self.borrow_mut() = new_val;
        }
    }
}
