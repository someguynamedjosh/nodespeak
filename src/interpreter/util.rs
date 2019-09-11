use crate::structure::{Expression, KnownData};
use std::convert::TryInto;

/// Expression must be a binary operator expression (add, equals, etc.) and A and B must be valid 
/// inputs for that expression. They cannot have different base types.
/// TODO: Implement arrays, and arrays of different sizes.
pub(super) fn compute_binary_expr(expression: &Expression, a: &KnownData, b: &KnownData) -> KnownData {
    match expression {
        Expression::Add(..) => match a {
            KnownData::Bool(..) => unimplemented!(),
            KnownData::Int(value) => KnownData::Int(
                value + b.require_int()
            ),
            KnownData::Float(value) => KnownData::Float(
                value + b.require_float()
            ),
            _ => unreachable!()
        }
        Expression::Subtract(..) => match a {
            KnownData::Bool(..) => unimplemented!(),
            KnownData::Int(value) => KnownData::Int(
                value - b.require_int()
            ),
            KnownData::Float(value) => KnownData::Float(
                value - b.require_float()
            ),
            _ => unreachable!()
        }
        Expression::Multiply(..) => match a {
            KnownData::Bool(..) => unimplemented!(),
            KnownData::Int(value) => KnownData::Int(
                value * b.require_int()
            ),
            KnownData::Float(value) => KnownData::Float(
                value * b.require_float()
            ),
            _ => unreachable!()
        }
        Expression::Divide(..) => match a {
            KnownData::Float(value) => KnownData::Float(
                value / b.require_float()
            ),
            _ => unreachable!()
        }
        Expression::IntDiv(..) => match a {
            KnownData::Int(value) => KnownData::Int(
                value / b.require_int()
            ),
            _ => unreachable!()
        }
        Expression::Modulo(..) => match a {
            KnownData::Bool(..) => unimplemented!(),
            KnownData::Int(value) => KnownData::Int(
                value % b.require_int()
            ),
            KnownData::Float(value) => KnownData::Float(
                value % b.require_float()
            ),
            _ => unreachable!()
        }
        Expression::Power(..) => match a {
            KnownData::Bool(..) => unimplemented!(),
            KnownData::Int(value) => KnownData::Int(
                i64::pow(*value, b.require_int().try_into().unwrap())
            ),
            KnownData::Float(value) => KnownData::Float(
                value.powf(b.require_float())
            ),
            _ => unreachable!()
        }
        Expression::And(..) => KnownData::Bool(a.require_bool() && b.require_bool()),
        Expression::Or(..) => KnownData::Bool(a.require_bool() || b.require_bool()),
        Expression::Xor(..) => KnownData::Bool(a.require_bool() != b.require_bool()),
        Expression::BAnd(..) => KnownData::Int(a.require_int() & b.require_int()),
        Expression::BOr(..) => KnownData::Int(a.require_int() | b.require_int()),
        Expression::BXor(..) => KnownData::Int(a.require_int() ^ b.require_int()),
        Expression::Equal(..) => match a {
            KnownData::Bool(value) => KnownData::Bool(*value == b.require_bool()),
            KnownData::Int(value) => KnownData::Bool(*value == b.require_int()),
            KnownData::Float(value) => KnownData::Bool(*value == b.require_float()),
            KnownData::DataType(value) => KnownData::Bool(value == b.require_data_type()),
            KnownData::Function(value) => KnownData::Bool(value == b.require_function()),
            KnownData::Array(value) => KnownData::Bool(value == b.require_array()),
            _ => unreachable!()
        }
        Expression::NotEqual(..) => match a {
            KnownData::Bool(value) => KnownData::Bool(*value != b.require_bool()),
            KnownData::Int(value) => KnownData::Bool(*value != b.require_int()),
            KnownData::Float(value) => KnownData::Bool(*value != b.require_float()),
            KnownData::DataType(value) => KnownData::Bool(value != b.require_data_type()),
            KnownData::Function(value) => KnownData::Bool(value != b.require_function()),
            KnownData::Array(value) => KnownData::Bool(value != b.require_array()),
            _ => unreachable!()
        }
        Expression::LessThan(..) => match a {
            KnownData::Int(value) => KnownData::Bool(*value < b.require_int()),
            KnownData::Float(value) => KnownData::Bool(*value < b.require_float()),
            _ => unreachable!()
        }
        Expression::GreaterThan(..) => match a {
            KnownData::Int(value) => KnownData::Bool(*value > b.require_int()),
            KnownData::Float(value) => KnownData::Bool(*value > b.require_float()),
            _ => unreachable!()
        }
        Expression::LessThanOrEqual(..) => match a {
            KnownData::Int(value) => KnownData::Bool(*value <= b.require_int()),
            KnownData::Float(value) => KnownData::Bool(*value <= b.require_float()),
            _ => unreachable!()
        }
        Expression::GreaterThanOrEqual(..) => match a {
            KnownData::Int(value) => KnownData::Bool(*value >= b.require_int()),
            KnownData::Float(value) => KnownData::Bool(*value >= b.require_float()),
            _ => unreachable!()
        }
        _ => unreachable!()
    }
}