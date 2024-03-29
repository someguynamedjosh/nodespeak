use std::collections::HashMap;

use itertools::Itertools;

use super::{
    type_arithmetic::calculate_type_arithmetic,
    type_compatibility::type_a_is_compatible_with_type_b, BuiltinOp, BuiltinType, LocalPtr,
    Statement, Value, ValuePtr,
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
        BuiltinOp::Cast => unreachable!(),
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
        BuiltinOp::Cast => unreachable!(),
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
        BuiltinOp::Cast => unreachable!(),
    }
}

#[derive(Clone, Debug)]
pub struct SimplificationContext {
    previous_blocks: Vec<HashMap<LocalPtr, ValuePtr>>,
    current_block: HashMap<LocalPtr, ValuePtr>,
}

impl SimplificationContext {
    pub fn new() -> Self {
        Self {
            previous_blocks: Vec::new(),
            current_block: HashMap::new(),
        }
    }

    fn start_new_block(&mut self) {
        self.previous_blocks
            .push(std::mem::take(&mut self.current_block));
    }

    pub fn finish(mut self) -> Vec<HashMap<LocalPtr, ValuePtr>> {
        self.start_new_block();
        self.previous_blocks
    }
}

impl ValuePtr {
    pub fn typee(&self) -> Value {
        match &*self.0.borrow() {
            Value::BuiltinType(typ) => match typ {
                BuiltinType::Int
                | BuiltinType::Float
                | BuiltinType::Bool
                | BuiltinType::Any
                | BuiltinType::Type
                | BuiltinType::Array { .. }
                | BuiltinType::InSet { .. }
                | BuiltinType::Function { .. }
                | BuiltinType::Malformed => Value::BuiltinType(BuiltinType::Type),
            },
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
                BuiltinOp::Not | BuiltinOp::Typeof | BuiltinOp::Cast => todo!(),
            },
            Value::ArrayLiteral {
                elements: _,
                dims: _,
            } => todo!(),
            Value::FloatLiteral(_) => Value::BuiltinType(BuiltinType::Float),
            Value::IntLiteral(_) => Value::BuiltinType(BuiltinType::Int),
            Value::BoolLiteral(_) => Value::BuiltinType(BuiltinType::Bool),
            Value::Local(local) => (local.typee.borrow()).clone(),
            Value::Function {
                inputs: _,
                outputs: _,
                locals: _,
                body: _,
            } => todo!(),
            Value::FunctionCall(base, args, _) => match &*base.borrow() {
                Value::BuiltinOp(BuiltinOp::Add)
                | Value::BuiltinOp(BuiltinOp::Sub)
                | Value::BuiltinOp(BuiltinOp::Mul)
                | Value::BuiltinOp(BuiltinOp::Div)
                | Value::BuiltinOp(BuiltinOp::Rem)
                | Value::BuiltinOp(BuiltinOp::Gt)
                | Value::BuiltinOp(BuiltinOp::Lt)
                | Value::BuiltinOp(BuiltinOp::Gte)
                | Value::BuiltinOp(BuiltinOp::Lte)
                | Value::BuiltinOp(BuiltinOp::Eq)
                | Value::BuiltinOp(BuiltinOp::Neq)
                | Value::BuiltinOp(BuiltinOp::And)
                | Value::BuiltinOp(BuiltinOp::Or)
                | Value::BuiltinOp(BuiltinOp::Xor) => Value::FunctionCall(
                    base.ptr_clone(),
                    args.iter().map(|x| ValuePtr::new(x.typee())).collect(),
                    0,
                ),
                Value::BuiltinOp(BuiltinOp::Cast) => args[0].borrow().clone(),
                _ => todo!(),
            },
        }
    }

    pub fn check_and_simplify(&self, ctx: &mut SimplificationContext) {
        let new_val = match &*self.0.borrow() {
            Value::BuiltinType(BuiltinType::Array { eltype, dims }) => {
                eltype.check_and_simplify(ctx);
                dims.iter().for_each(|x| x.check_and_simplify(ctx));
                if let Value::BuiltinType(BuiltinType::Array {
                    eltype: sub_eltype,
                    dims: sub_dims,
                }) = &*eltype.borrow()
                {
                    Some(Value::BuiltinType(BuiltinType::Array {
                        eltype: sub_eltype.ptr_clone(),
                        dims: [sub_dims.clone(), dims.clone()].concat(),
                    }))
                } else {
                    None
                }
            }
            Value::BuiltinType(BuiltinType::InSet { eltype, elements }) => {
                eltype.check_and_simplify(ctx);
                elements.iter().for_each(|x| x.check_and_simplify(ctx));
                None
            }
            Value::BuiltinType(BuiltinType::Function { inputs, outputs }) => {
                inputs.iter().for_each(|x| x.check_and_simplify(ctx));
                outputs.iter().for_each(|x| x.check_and_simplify(ctx));
                None
            }
            Value::FloatLiteral(_)
            | Value::IntLiteral(_)
            | Value::BoolLiteral(_)
            | Value::BuiltinType(_)
            | Value::BuiltinOp(_)
            | Value::Malformed => None,
            Value::ArrayLiteral { elements, dims } => {
                elements.iter().for_each(|x| x.check_and_simplify(ctx));
                dims.iter().for_each(|x| x.check_and_simplify(ctx));
                None
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
                    statement.check_and_simplify(&mut new_ctx);
                }
                for (target, base) in &new_ctx.current_block {
                    if !ctx.current_block.contains_key(target) {
                        let base_type = ValuePtr::new(base.typee());
                        let mut sub_ctx = SimplificationContext::new();
                        base_type.check_and_simplify(&mut sub_ctx);
                        target.typee.check_and_simplify(&mut sub_ctx);
                        if !type_a_is_compatible_with_type_b(&base_type, &target.typee) {
                            panic!("Invalid assignment");
                        }
                        new_body.push(Statement::Assignment {
                            base: base.ptr_clone(),
                            index: None,
                            target: target.ptr_clone(),
                        });
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
                base.check_and_simplify(ctx);
                args.iter().for_each(|x| x.check_and_simplify(ctx));
                if let Value::BuiltinOp(op) = &*base.borrow() {
                    let new_value = if args.len() == 2 {
                        let mut sub_ctx = SimplificationContext::new();
                        let mut args = args.clone().into_iter();
                        let lhs = args.next().unwrap();
                        lhs.check_and_simplify(ctx);
                        let lhs_type = ValuePtr::new(lhs.typee());
                        lhs_type.check_and_simplify(&mut sub_ctx);
                        let rhs = args.next().unwrap();
                        rhs.check_and_simplify(ctx);
                        let rhs_type = ValuePtr::new(rhs.typee());
                        rhs_type.check_and_simplify(&mut sub_ctx);
                        if op == &BuiltinOp::Cast {
                            if !type_a_is_compatible_with_type_b(
                                &lhs,
                                &ValuePtr::new(Value::BuiltinType(BuiltinType::Type)),
                            ) {
                                println!("{:#?}", lhs);
                                panic!("lhs is not a type");
                            }
                            if !type_a_is_compatible_with_type_b(&rhs_type, &lhs) {
                                panic!("rhs is not castable to lhs");
                            }
                        } else {
                            let combined_type = if let (
                                Value::BuiltinType(lhs_type),
                                Value::BuiltinType(rhs_type),
                            ) = (&*lhs_type.borrow(), &*rhs_type.borrow())
                            {
                                ValuePtr::new(calculate_type_arithmetic(
                                    *op,
                                    &[lhs_type.clone(), rhs_type.clone()],
                                ))
                            } else {
                                ValuePtr::new(Value::FunctionCall(
                                    ValuePtr::new(Value::BuiltinOp(*op)),
                                    vec![lhs_type.ptr_clone(), rhs_type.ptr_clone()],
                                    0,
                                ))
                            };
                            let mut sub_ctx = SimplificationContext::new();
                            combined_type.check_and_simplify(&mut sub_ctx);
                            if &*combined_type.borrow()
                                == &Value::BuiltinType(BuiltinType::Malformed)
                            {
                                panic!("Invalid binary operation");
                            }
                        }
                        let res = match (&*lhs.borrow(), &*rhs.borrow()) {
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
                            (Value::BuiltinType(lhs), Value::BuiltinType(rhs)) => {
                                Some(calculate_type_arithmetic(*op, &[lhs.clone(), rhs.clone()]))
                            }
                            _ => None,
                        };
                        res
                    } else if op == &BuiltinOp::Typeof {
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
                    assert_eq!(args.len(), inputs.len(), "Incorrect number of arguments.");
                    for (target, arg) in inputs.iter().zip(args.iter()) {
                        let arg_type = ValuePtr::new(arg.typee());
                        arg_type.check_and_simplify(&mut new_ctx);
                        let target_type = target.typee.deep_clone();
                        target_type.check_and_simplify(&mut new_ctx);
                        if !type_a_is_compatible_with_type_b(&arg_type, &target_type) {
                            println!("{:#?}", arg_type);
                            println!("{:#?}", target_type);
                            panic!("Invalid argument.");
                        }
                        new_ctx
                            .current_block
                            .insert(target.ptr_clone(), arg.ptr_clone());
                    }
                    for statement in body {
                        statement.check_and_simplify(&mut new_ctx);
                    }
                    let mut results = Vec::new();
                    for output in outputs {
                        if let Some(result) = new_ctx.current_block.get(output) {
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
                if let Some(value) = ctx.current_block.get(local) {
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

impl Statement {
    pub fn check_and_simplify(&self, ctx: &mut SimplificationContext) {
        match self {
            Self::Assignment {
                base,
                index,
                target,
            } => {
                base.check_and_simplify(ctx);
                if &*target.typee.borrow() != &Value::BuiltinType(BuiltinType::Any) {
                    let base_type = ValuePtr::new(base.typee());
                    let mut sub_ctx = SimplificationContext::new();
                    base_type.check_and_simplify(&mut sub_ctx);
                    target.typee.check_and_simplify(&mut sub_ctx);
                    let target_typee = if let Some(index) = index {
                        match &*target.typee.borrow() {
                            Value::BuiltinType(BuiltinType::Array { eltype, dims }) => {
                                if index.indices.len() < dims.len() {
                                    panic!("Not enough indices in assignment");
                                }
                                eltype.ptr_clone()
                            }
                            _ => todo!(),
                        }
                    } else {
                        target.typee.ptr_clone()
                    };
                    if !type_a_is_compatible_with_type_b(&base_type, &target_typee) {
                        println!("{:#?} => {:#?}", base_type, target.typee);
                        panic!("Invalid assignment");
                    }
                }
                ctx.current_block
                    .insert(target.ptr_clone(), base.ptr_clone());
            }
            Self::Declaration(local) => {
                local.typee.check_and_simplify(ctx);
            }
            Self::Noop => (),
        };
    }
}
