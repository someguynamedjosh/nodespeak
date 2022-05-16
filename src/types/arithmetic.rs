use itertools::Itertools;

use super::Type;
use crate::values::{Operation, Value, ValuePtr};

pub fn calculate_type_arithmetic(op: Operation, values: &[Type]) -> Value {
    fn binary(values: &[Type]) -> (&Type, &Type) {
        assert_eq!(values.len(), 2);
        let mut values = values.iter();
        let lhs = values.next().unwrap();
        let rhs = values.next().unwrap();
        (lhs, rhs)
    }
    let typ = match op {
        Operation::Add
        | Operation::Sub
        | Operation::Mul
        | Operation::Div
        | Operation::Gt
        | Operation::Lt
        | Operation::Gte
        | Operation::Lte
        | Operation::Eq
        | Operation::Neq
        | Operation::And
        | Operation::Or
        | Operation::Xor
        | Operation::Rem => {
            let (lhs, rhs) = binary(values);
            match (lhs, rhs) {
                (Type::Int, Type::Int) => Type::Int,
                (Type::Float, Type::Float)
                | (Type::Int, Type::Float)
                | (Type::Float, Type::Int) => Type::Float,
                (_, Type::Bool) | (Type::Bool, _) => Type::Malformed,
                (_, Type::Function { .. }) | (Type::Function { .. }, _) => Type::Malformed,
                (_, Type::Malformed) | (Type::Malformed, _) => Type::Malformed,
                (
                    Type::Array {
                        eltype: left_type,
                        dims: left_dims,
                    },
                    Type::Array {
                        eltype: right_type,
                        dims: right_dims,
                    },
                ) => {
                    let eltype = ValuePtr::new(Value::Operation(
                        op,
                        vec![left_type.ptr_clone(), right_type.ptr_clone()],
                    ));
                    broadcast_array_dims(eltype, left_dims, right_dims)
                }
                (
                    left_type,
                    Type::Array {
                        eltype: right_type,
                        dims: right_dims,
                    },
                ) => {
                    let left_type = ValuePtr::new(Value::TypeLiteral(left_type.clone()));
                    let eltype = ValuePtr::new(Value::Operation(
                        op,
                        vec![left_type, right_type.ptr_clone()],
                    ));
                    broadcast_array_dims(eltype, &[], right_dims)
                }
                (
                    Type::Array {
                        eltype: left_type,
                        dims: left_dims,
                    },
                    right_type,
                ) => {
                    let right_type = ValuePtr::new(Value::TypeLiteral(right_type.clone()));
                    let eltype = ValuePtr::new(Value::Operation(
                        op,
                        vec![left_type.ptr_clone(), right_type],
                    ));
                    broadcast_array_dims(eltype, left_dims, &[])
                }
                (
                    Type::InSet {
                        base_type: left_base,
                        allowed_values: left_values,
                    },
                    Type::InSet {
                        base_type: right_base,
                        allowed_values: right_values,
                    },
                ) => {
                    let combined_base = ValuePtr::new(Value::Operation(
                        op,
                        vec![left_base.ptr_clone(), right_base.ptr_clone()],
                    ));
                    let mut new_values = Vec::new();
                    for left in left_values {
                        for right in right_values {
                            new_values.push(ValuePtr::new(Value::Operation(
                                op,
                                vec![left.ptr_clone(), right.ptr_clone()],
                            )));
                        }
                    }
                    Type::InSet {
                        base_type: combined_base,
                        allowed_values: new_values,
                    }
                }
                (
                    Type::InSet {
                        base_type: left_base,
                        ..
                    },
                    right,
                ) => {
                    return Value::Operation(
                        op,
                        vec![
                            left_base.ptr_clone(),
                            ValuePtr::new(Value::TypeLiteral(right.clone())),
                        ],
                    );
                }
                (
                    left,
                    Type::InSet {
                        base_type: right_base,
                        ..
                    },
                ) => {
                    return Value::Operation(
                        op,
                        vec![
                            ValuePtr::new(Value::TypeLiteral(left.clone())),
                            right_base.ptr_clone(),
                        ],
                    );
                }
            }
        }
        Operation::Not => {
            assert_eq!(values.len(), 1);
            let mut values = values.iter();
            let base = values.next().unwrap();
            base.clone()
        }
    };
    Value::TypeLiteral(typ)
}

fn broadcast_array_dims(eltype: ValuePtr, left_dims: &[ValuePtr], right_dims: &[ValuePtr]) -> Type {
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
            return Type::Malformed;
        }
    }
    Type::Array {
        eltype,
        dims: new_dims,
    }
}

fn broadcast_dim(left_value: &ValuePtr, right_value: &ValuePtr) -> Option<ValuePtr> {
    match (&**left_value, &**right_value) {
        (&Value::IntLiteral(1), _) => Some(right_value.ptr_clone()),
        (_, &Value::IntLiteral(1)) => Some(left_value.ptr_clone()),
        (&Value::IntLiteral(left), &Value::IntLiteral(right)) if left == right => {
            Some(left_value.ptr_clone())
        }
        (Value::FunctionParameter(left), Value::FunctionParameter(right)) if left == right => {
            Some(left_value.ptr_clone())
        }
        (left, right) if left == right => Some(left_value.ptr_clone()),
        _ => None,
    }
}
