use super::{BaseType, DataType};
use crate::problem::CompileProblem;
use crate::resolved::structure as o;
use crate::shared as s;
use crate::vague::structure as i;

use std::convert::TryInto;

enum IndexMethod {
    Basic,  // Literally use the current index. (value[index])
    Single, // Always use the index 0. (value[0])
    Ignore, // Do not index. (value)
}

/// Expression must be a binary operator expression (add, equals, etc.) and A and B must be valid
/// inputs for that expression. They cannot have different base types.
/// TODO: Implement arrays, and arrays of different sizes.
pub(super) fn compute_binary_operation(
    a: &i::KnownData,
    operator: i::BinaryOperator,
    b: &i::KnownData,
) -> i::KnownData {
    if let i::KnownData::Array(array_a) = a {
        if let i::KnownData::Array(array_b) = b {
            let a_size = array_a.len();
            let b_size = array_b.len();

            let (inc_a, inc_b) = if a_size == b_size {
                (true, true)
            } else if a_size == 1 {
                (false, true)
            } else if b_size == 1 {
                (true, false)
            } else {
                panic!("TODO: nice error, invalid inflation.");
            };

            let result_size = a_size.max(b_size);
            let mut result_items = Vec::with_capacity(result_size);
            let mut a_index = 0;
            let mut b_index = 0;
            for index in 0..result_size {
                // Do some math, finally.
                result_items.push(compute_binary_operation(
                    &array_a[a_index],
                    operator,
                    &array_b[b_index],
                ));

                // Update the index for the next go-around.
                if inc_a {
                    a_index += 1;
                }
                if inc_b {
                    b_index += 1;
                }
            }
            i::KnownData::Array(result_items)
        } else {
            let a_size = array_a.len();
            let mut items = Vec::with_capacity(a_size);
            for a_item in array_a {
                items.push(compute_binary_operation_impl(a_item, operator, b));
            }
            i::KnownData::Array(items)
        }
    } else {
        if let i::KnownData::Array(array_b) = b {
            let b_size = array_b.len();
            let mut items = Vec::with_capacity(b_size);
            for b_item in array_b {
                items.push(compute_binary_operation_impl(a, operator, b_item));
            }
            i::KnownData::Array(items)
        } else {
            compute_binary_operation_impl(a, operator, b)
        }
    }
}

fn compute_binary_operation_impl(
    a: &i::KnownData,
    operator: i::BinaryOperator,
    b: &i::KnownData,
) -> i::KnownData {
    match operator {
        i::BinaryOperator::Add => match a {
            i::KnownData::Bool(..) => unimplemented!(),
            i::KnownData::Int(value) => i::KnownData::Int(value + b.require_int()),
            i::KnownData::Float(value) => i::KnownData::Float(value + b.require_float()),
            _ => unreachable!(),
        },
        i::BinaryOperator::Subtract => match a {
            i::KnownData::Bool(..) => unimplemented!(),
            i::KnownData::Int(value) => i::KnownData::Int(value - b.require_int()),
            i::KnownData::Float(value) => i::KnownData::Float(value - b.require_float()),
            _ => unreachable!(),
        },
        i::BinaryOperator::Multiply => match a {
            i::KnownData::Bool(..) => unimplemented!(),
            i::KnownData::Int(value) => i::KnownData::Int(value * b.require_int()),
            i::KnownData::Float(value) => i::KnownData::Float(value * b.require_float()),
            _ => unreachable!(),
        },
        i::BinaryOperator::Divide => match a {
            i::KnownData::Float(value) => i::KnownData::Float(value / b.require_float()),
            i::KnownData::Int(value) => i::KnownData::Int(value / b.require_int()),
            _ => unreachable!(),
        },
        i::BinaryOperator::Modulo => match a {
            i::KnownData::Bool(..) => unimplemented!(),
            i::KnownData::Int(value) => i::KnownData::Int(value % b.require_int()),
            i::KnownData::Float(value) => i::KnownData::Float(value % b.require_float()),
            _ => unreachable!(),
        },
        i::BinaryOperator::Power => match a {
            i::KnownData::Bool(..) => unimplemented!(),
            i::KnownData::Int(value) => {
                i::KnownData::Int(i64::pow(*value, b.require_int().try_into().unwrap()))
            }
            i::KnownData::Float(value) => i::KnownData::Float(value.powf(b.require_float())),
            _ => unreachable!(),
        },
        i::BinaryOperator::And => i::KnownData::Bool(a.require_bool() && b.require_bool()),
        i::BinaryOperator::Or => i::KnownData::Bool(a.require_bool() || b.require_bool()),
        i::BinaryOperator::Xor => i::KnownData::Bool(a.require_bool() != b.require_bool()),
        i::BinaryOperator::BAnd => i::KnownData::Int(a.require_int() & b.require_int()),
        i::BinaryOperator::BOr => i::KnownData::Int(a.require_int() | b.require_int()),
        i::BinaryOperator::BXor => i::KnownData::Int(a.require_int() ^ b.require_int()),
        i::BinaryOperator::Equal => match a {
            i::KnownData::Bool(value) => i::KnownData::Bool(*value == b.require_bool()),
            i::KnownData::Int(value) => i::KnownData::Bool(*value == b.require_int()),
            i::KnownData::Float(value) => i::KnownData::Bool(*value == b.require_float()),
            i::KnownData::DataType(value) => i::KnownData::Bool(value == b.require_data_type()),
            i::KnownData::Function(value) => i::KnownData::Bool(value == b.require_function()),
            i::KnownData::Array(value) => i::KnownData::Bool(value == b.require_array()),
            _ => unreachable!(),
        },
        i::BinaryOperator::NotEqual => match a {
            i::KnownData::Bool(value) => i::KnownData::Bool(*value != b.require_bool()),
            i::KnownData::Int(value) => i::KnownData::Bool(*value != b.require_int()),
            i::KnownData::Float(value) => i::KnownData::Bool(*value != b.require_float()),
            i::KnownData::DataType(value) => i::KnownData::Bool(value != b.require_data_type()),
            i::KnownData::Function(value) => i::KnownData::Bool(value != b.require_function()),
            i::KnownData::Array(value) => i::KnownData::Bool(value != b.require_array()),
            _ => unreachable!(),
        },
        i::BinaryOperator::LessThan => match a {
            i::KnownData::Int(value) => i::KnownData::Bool(*value < b.require_int()),
            i::KnownData::Float(value) => i::KnownData::Bool(*value < b.require_float()),
            _ => unreachable!(),
        },
        i::BinaryOperator::GreaterThan => match a {
            i::KnownData::Int(value) => i::KnownData::Bool(*value > b.require_int()),
            i::KnownData::Float(value) => i::KnownData::Bool(*value > b.require_float()),
            _ => unreachable!(),
        },
        i::BinaryOperator::LessThanOrEqual => match a {
            i::KnownData::Int(value) => i::KnownData::Bool(*value <= b.require_int()),
            i::KnownData::Float(value) => i::KnownData::Bool(*value <= b.require_float()),
            _ => unreachable!(),
        },
        i::BinaryOperator::GreaterThanOrEqual => match a {
            i::KnownData::Int(value) => i::KnownData::Bool(*value >= b.require_int()),
            i::KnownData::Float(value) => i::KnownData::Bool(*value >= b.require_float()),
            _ => unreachable!(),
        },
    }
}

fn biggest_scalar(a: &BaseType, b: &BaseType) -> Result<BaseType, ()> {
    // BCT rule 1
    if a == &BaseType::Automatic {
        Result::Ok(b.clone())
    } else if b == &BaseType::Automatic {
        Result::Ok(a.clone())
    } else if a == b {
        Result::Ok(a.clone())
    } else {
        Result::Err(())
    }
}

/// Returns Result::Err if there is no biggest type.
pub(super) fn biggest_type(a: &DataType, b: &DataType) -> Result<DataType, ()> {
    // BCT rule 1
    if a.is_specific_scalar(&BaseType::Automatic) {
        Result::Ok(b.clone())
    } else if b.is_specific_scalar(&BaseType::Automatic) {
        Result::Ok(a.clone())
    // BCT rule 2
    } else if a.equivalent(b) {
        Result::Ok(a.clone())
    // BCT rules 3 - 5
    } else if a.is_array() || b.is_array() {
        let a_dims = a.borrow_dimensions().clone();
        let b_dims = b.borrow_dimensions().clone();
        let mut final_dims = Vec::new();
        for index in 0..a_dims.len().max(b_dims.len()) {
            // BCT rule 5
            if index >= a_dims.len() {
                final_dims.push(b_dims[index]);
            } else if index >= b_dims.len() {
                final_dims.push(a_dims[index]);
            // BCT rule 3
            } else if a_dims[index] == b_dims[index] {
                final_dims.push(a_dims[index]);
            // BCT rule 4
            } else if a_dims[index] == 1 {
                final_dims.push(b_dims[index]);
            } else if b_dims[index] == 1 {
                final_dims.push(a_dims[index]);
            } else {
                return Result::Err(());
            }
        }
        let base_type = biggest_scalar(a.borrow_base(), b.borrow_base())?;
        Result::Ok(DataType::array(base_type, final_dims))
    } else {
        Result::Err(())
    }
}

pub(super) fn inflate(
    expr: o::Expression,
    from: &DataType,
    to: &DataType,
) -> Result<o::Expression, CompileProblem> {
    // TODO: Nice error if array dimension not int.
    let from_dims = from.borrow_dimensions().clone();
    let to_dims = to.borrow_dimensions().clone();
    let mut final_dims = Vec::new();
    for index in 0..from_dims.len().max(to_dims.len()) {
        if index >= to_dims.len() {
            panic!("TODO: nice error, cannot inflate to smaller dimension array.");
        } else if index >= from_dims.len() {
            final_dims.push((to_dims[index], s::ProxyMode::Discard));
        } else if to_dims[index] == from_dims[index] {
            final_dims.push((to_dims[index], s::ProxyMode::Keep));
        } else if from_dims[index] == 1 {
            final_dims.push((to_dims[index], s::ProxyMode::Collapse));
        } else {
            panic!("TODO: nice error, invalid inflation.");
        }
    }
    Result::Ok(o::Expression::Proxy {
        base: Box::new(expr),
        dimensions: final_dims,
    })
}

pub(super) fn resolve_known_data(input: &i::KnownData) -> Result<o::KnownData, ()> {
    Result::Ok(match input {
        i::KnownData::Bool(value) => o::KnownData::Bool(*value),
        i::KnownData::Int(value) => o::KnownData::Int(*value),
        i::KnownData::Float(value) => o::KnownData::Float(*value),
        i::KnownData::Array(old_data) => {
            let mut items = Vec::with_capacity(old_data.len());
            for old_item in old_data {
                items.push(resolve_known_data(old_item)?);
            }
            o::KnownData::Array(items)
        }
        i::KnownData::DataType(..)
        | i::KnownData::Function(..)
        | i::KnownData::Unknown
        | i::KnownData::Void => return Result::Err(()),
    })
}

pub(super) fn resolve_operator(operator: i::BinaryOperator) -> o::BinaryOperator {
    match operator {
        i::BinaryOperator::Add => o::BinaryOperator::Add,
        i::BinaryOperator::And => o::BinaryOperator::And,
        i::BinaryOperator::BAnd => o::BinaryOperator::BAnd,
        i::BinaryOperator::BOr => o::BinaryOperator::BOr,
        i::BinaryOperator::BXor => o::BinaryOperator::BXor,
        i::BinaryOperator::Divide => o::BinaryOperator::Divide,
        i::BinaryOperator::Equal => o::BinaryOperator::Equal,
        i::BinaryOperator::GreaterThan => o::BinaryOperator::GreaterThan,
        i::BinaryOperator::GreaterThanOrEqual => o::BinaryOperator::GreaterThanOrEqual,
        i::BinaryOperator::LessThan => o::BinaryOperator::LessThan,
        i::BinaryOperator::LessThanOrEqual => o::BinaryOperator::LessThanOrEqual,
        i::BinaryOperator::Modulo => o::BinaryOperator::Modulo,
        i::BinaryOperator::Multiply => o::BinaryOperator::Multiply,
        i::BinaryOperator::NotEqual => o::BinaryOperator::NotEqual,
        i::BinaryOperator::Or => o::BinaryOperator::Or,
        i::BinaryOperator::Power => o::BinaryOperator::Power,
        i::BinaryOperator::Subtract => o::BinaryOperator::Subtract,
        i::BinaryOperator::Xor => o::BinaryOperator::Xor,
    }
}
