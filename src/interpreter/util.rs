use crate::structure::{BinaryOperator, KnownData};
use std::convert::TryInto;

/// Expression must be a binary operator expression (add, equals, etc.) and A and B must be valid 
/// inputs for that expression. They cannot have different base types.
/// TODO: Implement arrays, and arrays of different sizes.
pub(super) fn compute_binary_operation(a: &KnownData, operator: BinaryOperator, b: &KnownData) -> KnownData {
    match operator {
        BinaryOperator::Add => match a {
            KnownData::Bool(..) => unimplemented!(),
            KnownData::Int(value) => KnownData::Int(
                value + b.require_int()
            ),
            KnownData::Float(value) => KnownData::Float(
                value + b.require_float()
            ),
            _ => unreachable!()
        }
        BinaryOperator::Subtract => match a {
            KnownData::Bool(..) => unimplemented!(),
            KnownData::Int(value) => KnownData::Int(
                value - b.require_int()
            ),
            KnownData::Float(value) => KnownData::Float(
                value - b.require_float()
            ),
            _ => unreachable!()
        }
        BinaryOperator::Multiply => match a {
            KnownData::Bool(..) => unimplemented!(),
            KnownData::Int(value) => KnownData::Int(
                value * b.require_int()
            ),
            KnownData::Float(value) => KnownData::Float(
                value * b.require_float()
            ),
            _ => unreachable!()
        }
        BinaryOperator::Divide => match a {
            KnownData::Float(value) => KnownData::Float(
                value / b.require_float()
            ),
            _ => unreachable!()
        }
        BinaryOperator::IntDiv => match a {
            KnownData::Int(value) => KnownData::Int(
                value / b.require_int()
            ),
            _ => unreachable!()
        }
        BinaryOperator::Modulo => match a {
            KnownData::Bool(..) => unimplemented!(),
            KnownData::Int(value) => KnownData::Int(
                value % b.require_int()
            ),
            KnownData::Float(value) => KnownData::Float(
                value % b.require_float()
            ),
            _ => unreachable!()
        }
        BinaryOperator::Power => match a {
            KnownData::Bool(..) => unimplemented!(),
            KnownData::Int(value) => KnownData::Int(
                i64::pow(*value, b.require_int().try_into().unwrap())
            ),
            KnownData::Float(value) => KnownData::Float(
                value.powf(b.require_float())
            ),
            _ => unreachable!()
        }
        BinaryOperator::And => KnownData::Bool(a.require_bool() && b.require_bool()),
        BinaryOperator::Or => KnownData::Bool(a.require_bool() || b.require_bool()),
        BinaryOperator::Xor => KnownData::Bool(a.require_bool() != b.require_bool()),
        BinaryOperator::BAnd => KnownData::Int(a.require_int() & b.require_int()),
        BinaryOperator::BOr => KnownData::Int(a.require_int() | b.require_int()),
        BinaryOperator::BXor => KnownData::Int(a.require_int() ^ b.require_int()),
        BinaryOperator::Equal => match a {
            KnownData::Bool(value) => KnownData::Bool(*value == b.require_bool()),
            KnownData::Int(value) => KnownData::Bool(*value == b.require_int()),
            KnownData::Float(value) => KnownData::Bool(*value == b.require_float()),
            KnownData::DataType(value) => KnownData::Bool(value == b.require_data_type()),
            KnownData::Function(value) => KnownData::Bool(value == b.require_function()),
            KnownData::Array(value) => KnownData::Bool(value == b.require_array()),
            _ => unreachable!()
        }
        BinaryOperator::NotEqual => match a {
            KnownData::Bool(value) => KnownData::Bool(*value != b.require_bool()),
            KnownData::Int(value) => KnownData::Bool(*value != b.require_int()),
            KnownData::Float(value) => KnownData::Bool(*value != b.require_float()),
            KnownData::DataType(value) => KnownData::Bool(value != b.require_data_type()),
            KnownData::Function(value) => KnownData::Bool(value != b.require_function()),
            KnownData::Array(value) => KnownData::Bool(value != b.require_array()),
            _ => unreachable!()
        }
        BinaryOperator::LessThan => match a {
            KnownData::Int(value) => KnownData::Bool(*value < b.require_int()),
            KnownData::Float(value) => KnownData::Bool(*value < b.require_float()),
            _ => unreachable!()
        }
        BinaryOperator::GreaterThan => match a {
            KnownData::Int(value) => KnownData::Bool(*value > b.require_int()),
            KnownData::Float(value) => KnownData::Bool(*value > b.require_float()),
            _ => unreachable!()
        }
        BinaryOperator::LessThanOrEqual => match a {
            KnownData::Int(value) => KnownData::Bool(*value <= b.require_int()),
            KnownData::Float(value) => KnownData::Bool(*value <= b.require_float()),
            _ => unreachable!()
        }
        BinaryOperator::GreaterThanOrEqual => match a {
            KnownData::Int(value) => KnownData::Bool(*value >= b.require_int()),
            KnownData::Float(value) => KnownData::Bool(*value >= b.require_float()),
            _ => unreachable!()
        }
        _ => unreachable!()
    }
}