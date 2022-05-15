use itertools::Itertools;

use super::Type;
use crate::values::{Operation, Value, ValuePtr};

pub fn calculate_type_arithmetic(op: Operation, values: &[Type]) -> Type {
    fn binary(values: &[Type]) -> (&Type, &Type) {
        assert_eq!(values.len(), 2);
        let mut values = values.iter();
        let lhs = values.next().unwrap();
        let rhs = values.next().unwrap();
        (lhs, rhs)
    }
    match op {
        Operation::Add => {
            let (lhs, rhs) = binary(values);
            match (lhs, rhs) {
                (Type::Int, Type::Int) => Type::Int,
                (Type::Float, Type::Float)
                | (Type::Int, Type::Float)
                | (Type::Float, Type::Int) => Type::Float,
                (_, Type::Bool) | (Type::Bool, _) => Type::Never,
                (_, Type::Function { .. }) | (Type::Function { .. }, _) => Type::Never,
                (_, Type::Never) | (Type::Never, _) => Type::Never,
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
                    todo!()
                }
                _ => todo!(),
            }
        }
        _ => todo!(),
    }
}

fn broadcast_array_dims(eltype: Type, left_dims: &[ValuePtr], right_dims: &[ValuePtr]) -> Type {
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
    todo!()
}

fn broadcast_dim(left_value: ValuePtr, right_value: ValuePtr) -> Option<ValuePtr> {
    match (&*left_value, &*right_value) {
        (&Value::IntLiteral(left), &Value::IntLiteral(right)) => {
            if left == 1 {
                Some(right_value)
            } else if right == 1 {
                Some(left_value)
            } else if left == right {
                Some(left_value)
            } else {
                None
            }
        }
        (&Value::FloatLiteral(..), _) 
        | (_, &Value::FloatLiteral(..)) 
        | (&Value::BoolLiteral(..), _) 
        | (_, &Value::BoolLiteral(..)) 
        => None,
        _ => todo!(),
    }
}
