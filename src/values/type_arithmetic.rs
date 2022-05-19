use itertools::Itertools;

use super::{simplify::SimplificationContext, BuiltinType};
use crate::values::{BuiltinOp, Value, ValuePtr};

fn call(op: BuiltinOp, args: Vec<ValuePtr>) -> ValuePtr {
    let mut ctx = SimplificationContext::new();
    let ptr = ValuePtr::new(Value::FunctionCall(
        ValuePtr::new(Value::BuiltinOp(op)),
        args,
        0,
    ));
    ptr.check_and_simplify(&mut ctx);
    ptr
}

pub fn calculate_type_arithmetic(op: BuiltinOp, values: &[BuiltinType]) -> Value {
    fn binary(values: &[BuiltinType]) -> (&BuiltinType, &BuiltinType) {
        assert_eq!(values.len(), 2);
        let mut values = values.iter();
        let lhs = values.next().unwrap();
        let rhs = values.next().unwrap();
        (lhs, rhs)
    }
    let typ = match op {
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
        | BuiltinOp::Xor => {
            let (lhs, rhs) = binary(values);
            match (lhs, rhs) {
                (BuiltinType::Any, _) => BuiltinType::Any,
                (_, BuiltinType::Any) => BuiltinType::Any,
                (BuiltinType::Int, BuiltinType::Int) => BuiltinType::Int,
                (BuiltinType::Float, BuiltinType::Float)
                | (BuiltinType::Int, BuiltinType::Float)
                | (BuiltinType::Float, BuiltinType::Int) => BuiltinType::Float,
                (_, BuiltinType::Bool) | (BuiltinType::Bool, _) => BuiltinType::Malformed,
                (_, BuiltinType::Function { .. }) | (BuiltinType::Function { .. }, _) => {
                    BuiltinType::Malformed
                }
                (_, BuiltinType::Malformed) | (BuiltinType::Malformed, _) => BuiltinType::Malformed,
                (
                    BuiltinType::Array {
                        eltype: left_type,
                        dims: left_dims,
                    },
                    BuiltinType::Array {
                        eltype: right_type,
                        dims: right_dims,
                    },
                ) => {
                    let eltype = call(op, vec![left_type.ptr_clone(), right_type.ptr_clone()]);
                    broadcast_array_type(eltype, left_dims, right_dims)
                }
                (
                    left_type,
                    BuiltinType::Array {
                        eltype: right_type,
                        dims: right_dims,
                    },
                ) => {
                    let left_type = ValuePtr::new(Value::BuiltinType(left_type.clone()));
                    let eltype = call(op, vec![left_type, right_type.ptr_clone()]);
                    broadcast_array_type(eltype, &[], right_dims)
                }
                (
                    BuiltinType::Array {
                        eltype: left_type,
                        dims: left_dims,
                    },
                    right_type,
                ) => {
                    let right_type = ValuePtr::new(Value::BuiltinType(right_type.clone()));
                    let eltype = call(op, vec![left_type.ptr_clone(), right_type]);
                    broadcast_array_type(eltype, left_dims, &[])
                }
                (
                    BuiltinType::InSet {
                        eltype: left_eltype,
                        elements: left_elements,
                    },
                    BuiltinType::InSet {
                        eltype: right_eltype,
                        elements: right_elements,
                    },
                ) => {
                    let combined_eltype =
                        call(op, vec![left_eltype.ptr_clone(), right_eltype.ptr_clone()]);
                    let mut new_elements = Vec::new();
                    for left in left_elements {
                        for right in right_elements {
                            new_elements.push(call(op, vec![left.ptr_clone(), right.ptr_clone()]));
                        }
                    }
                    BuiltinType::InSet {
                        eltype: combined_eltype,
                        elements: new_elements,
                    }
                }
                (
                    BuiltinType::InSet {
                        eltype: left_eltype,
                        ..
                    },
                    right,
                ) => {
                    return Value::FunctionCall(
                        ValuePtr::new(Value::BuiltinOp(op)),
                        vec![
                            left_eltype.ptr_clone(),
                            ValuePtr::new(Value::BuiltinType(right.clone())),
                        ],
                        0,
                    );
                }
                (
                    left,
                    BuiltinType::InSet {
                        eltype: right_eltype,
                        ..
                    },
                ) => {
                    return Value::FunctionCall(
                        ValuePtr::new(Value::BuiltinOp(op)),
                        vec![
                            ValuePtr::new(Value::BuiltinType(left.clone())),
                            right_eltype.ptr_clone(),
                        ],
                        0,
                    );
                }
                (BuiltinType::Type, BuiltinType::Type) => BuiltinType::Type,
                (BuiltinType::Type, _) | (_, BuiltinType::Type) => BuiltinType::Malformed,
            }
        }
        BuiltinOp::Not => {
            assert_eq!(values.len(), 1);
            let mut values = values.iter();
            let base = values.next().unwrap();
            base.clone()
        }
        BuiltinOp::Typeof => BuiltinType::Type,
        BuiltinOp::Cast => {
            panic!("Must be handled elsewhere as the result depends on the *value* of the first argument and not its type.")
        }
    };
    Value::BuiltinType(typ)
}

fn broadcast_array_type(
    eltype: ValuePtr,
    left_dims: &[ValuePtr],
    right_dims: &[ValuePtr],
) -> BuiltinType {
    if let Some(dims) = broadcast_array_dims(left_dims, right_dims) {
        BuiltinType::Array { eltype, dims }
    } else {
        BuiltinType::Malformed
    }
}

pub fn broadcast_array_dims(
    left_dims: &[ValuePtr],
    right_dims: &[ValuePtr],
) -> Option<Vec<ValuePtr>> {
    let one = ValuePtr::new(Value::IntLiteral(1));
    let mut left_dims = left_dims.iter().collect_vec();
    let mut right_dims = right_dims.iter().collect_vec();
    // For each dimension that only the right has...
    for _ in 0..right_dims.len().saturating_sub(left_dims.len()) {
        left_dims.push(&one);
    }
    // For each dimension that only the left has...
    for _ in 0..left_dims.len().saturating_sub(right_dims.len()) {
        right_dims.push(&one);
    }
    let mut new_dims = Vec::new();
    for (left, right) in left_dims.into_iter().zip(right_dims.into_iter()) {
        if let Some(dim) = broadcast_dim(left, right) {
            new_dims.push(dim);
        } else {
            return None;
        }
    }
    Some(new_dims)
}

fn broadcast_dim(left_value: &ValuePtr, right_value: &ValuePtr) -> Option<ValuePtr> {
    match (&*left_value.borrow(), &*right_value.borrow()) {
        (&Value::IntLiteral(1), _) => Some(right_value.ptr_clone()),
        (_, &Value::IntLiteral(1)) => Some(left_value.ptr_clone()),
        (&Value::IntLiteral(left), &Value::IntLiteral(right)) if left == right => {
            Some(left_value.ptr_clone())
        }
        (Value::Local(left), Value::Local(right)) if left == right => Some(left_value.ptr_clone()),
        (left, right) if left == right => Some(left_value.ptr_clone()),
        _ => {
            let ltype = left_value.typee();
            let rtype = right_value.typee();
            match (ltype, rtype) {
                (
                    Value::BuiltinType(BuiltinType::Int),
                    Value::BuiltinType(BuiltinType::InSet { elements, .. }),
                ) if elements.len() == 1 && &*elements[0].borrow() == &Value::IntLiteral(1) => {
                    Some(left_value.ptr_clone())
                }
                (
                    Value::BuiltinType(BuiltinType::InSet { elements, .. }),
                    Value::BuiltinType(BuiltinType::Int),
                ) if elements.len() == 1 && &*elements[0].borrow() == &Value::IntLiteral(1) => {
                    Some(right_value.ptr_clone())
                }
                (
                    Value::BuiltinType(BuiltinType::InSet { elements: lhs, .. }),
                    Value::BuiltinType(BuiltinType::InSet { elements: rhs, .. }),
                ) if lhs.len() == 1 && rhs.len() == 1 => broadcast_dim(&lhs[0], &rhs[0]),
                (
                    Value::BuiltinType(BuiltinType::InSet { elements: lhs, .. }),
                    Value::BuiltinType(BuiltinType::InSet { elements: rhs, .. }),
                ) if lhs.len() == 2 && rhs.len() == 1 => {
                    let a = broadcast_dim(&lhs[0], &rhs[0]);
                    let b = broadcast_dim(&lhs[1], &rhs[0]);
                    if let (Some(a), Some(b)) = (a, b) {
                        if a == b {
                            Some(right_value.ptr_clone())
                        } else {
                            Some(left_value.ptr_clone())
                        }
                    } else {
                        None
                    }
                }
                (
                    Value::BuiltinType(BuiltinType::InSet { elements: lhs, .. }),
                    Value::BuiltinType(BuiltinType::InSet { elements: rhs, .. }),
                ) if lhs.len() == 1 && rhs.len() == 2 => {
                    let a = broadcast_dim(&lhs[0], &rhs[0]);
                    let b = broadcast_dim(&lhs[0], &rhs[1]);
                    if let (Some(a), Some(b)) = (a, b) {
                        if a == b {
                            Some(left_value.clone())
                        } else {
                            Some(right_value.clone())
                        }
                    } else {
                        None
                    }
                }
                (
                    Value::BuiltinType(BuiltinType::InSet { elements: lhs, .. }),
                    Value::BuiltinType(BuiltinType::InSet { elements: rhs, .. }),
                ) if lhs.len() == 2 && rhs.len() == 2 => {
                    let a00 = broadcast_dim(&lhs[0], &rhs[0]);
                    let a01 = broadcast_dim(&lhs[0], &rhs[1]);
                    let a10 = broadcast_dim(&lhs[1], &rhs[0]);
                    let a11 = broadcast_dim(&lhs[1], &rhs[1]);
                    if let (Some(a00), Some(a01), Some(a10), Some(a11)) = (a00, a01, a10, a11) {
                        if a00 == a01 && a01 == lhs[0] && a10 == a11 && a11 == lhs[1] {
                            Some(left_value.ptr_clone())
                        } else if a00 == a10 && a10 == rhs[0] && a01 == a11 && a11 == rhs[1] {
                            Some(right_value.ptr_clone())
                        } else {
                            Some(call(
                                BuiltinOp::Max,
                                vec![left_value.ptr_clone(), right_value.ptr_clone()],
                            ))
                        }
                    } else {
                        None
                    }
                }
                _ => None,
            }
        }
    }
}
