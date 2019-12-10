use super::{problems, BaseType, Content, DataType, ScopeSimplifier, SimplifiedExpression};
use crate::problem::{CompileProblem, FilePosition};
use crate::resolved::structure as o;
use crate::util::NVec;
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
            let a_dims = array_a.borrow_dimensions().clone();
            let b_dims = array_b.borrow_dimensions().clone();
            let order = a_dims.len().max(b_dims.len());

            // Combine all the largest dimensions of each array into a single list of dimensions.
            let mut ab_dims = Vec::new();
            for index in 0..order {
                let mut ab_dim = 1;
                if index < a_dims.len() {
                    ab_dim = ab_dim.max(a_dims[index]);
                }
                if index < b_dims.len() {
                    ab_dim = ab_dim.max(b_dims[index]);
                }
                ab_dims.push(ab_dim);
            }

            // Considering the dimensions of the operators and the dimensions of the output, figure
            // out how to access everything.
            let mut a_index_methods = Vec::new();
            let mut b_index_methods = Vec::new();
            for index in 0..order {
                if index < a_dims.len() {
                    if a_dims[index] == ab_dims[index] {
                        a_index_methods.push(IndexMethod::Basic);
                    } else if a_dims[index] == 1 {
                        a_index_methods.push(IndexMethod::Single);
                    } else {
                        panic!("TODO: nice error, invalid inflation.");
                    }
                } else {
                    a_index_methods.push(IndexMethod::Ignore);
                }
                if index < b_dims.len() {
                    if b_dims[index] == ab_dims[index] {
                        b_index_methods.push(IndexMethod::Basic);
                    } else if b_dims[index] == 1 {
                        b_index_methods.push(IndexMethod::Single);
                    } else {
                        panic!("TODO: nice error, invalid inflation.");
                    }
                } else {
                    b_index_methods.push(IndexMethod::Ignore);
                }
            }

            let mut ab_index: Vec<_> = ab_dims.iter().map(|_| 0).collect();
            let mut a_index: Vec<_> = a_dims.iter().map(|_| 0).collect();
            let mut b_index: Vec<_> = b_dims.iter().map(|_| 0).collect();
            let mut result_items = Vec::new();
            'main_loop: loop {
                // Update a_index and b_index.
                for (dimension, index) in ab_index.iter().enumerate() {
                    match a_index_methods[dimension] {
                        IndexMethod::Basic => a_index[dimension] = *index,
                        IndexMethod::Single => a_index[dimension] = 1,
                        IndexMethod::Ignore => (),
                    }
                    match b_index_methods[dimension] {
                        IndexMethod::Basic => b_index[dimension] = *index,
                        IndexMethod::Single => b_index[dimension] = 1,
                        IndexMethod::Ignore => (),
                    }
                }
                // Do some math, finally.
                result_items.push(compute_binary_operation(
                    array_a.borrow_item(&a_index),
                    operator,
                    array_b.borrow_item(&b_index),
                ));

                // Update the index for the next go-around.
                let last_dimension = ab_index.len() - 1;
                ab_index[last_dimension] += 1;
                // Roll over that addition so that we are actually accessing the next element, not
                // just the [][][][][next] element.
                for dimension in (0..ab_index.len()).rev() {
                    if ab_index[dimension] < ab_dims[dimension] {
                        continue;
                    }
                    if dimension > 0 {
                        ab_index[dimension] = 0;
                        ab_index[dimension - 1] += 1;
                    } else {
                        // We have moved past the range of the biggest dimension, therefore there
                        // is no more array to iterate over.
                        break 'main_loop;
                    }
                }
            }
            i::KnownData::Array(NVec::from_vec_and_dims(result_items, ab_dims))
        } else {
            let dimensions = array_a.borrow_dimensions().clone();
            let mut items = Vec::with_capacity(array_a.borrow_all_items().len());
            for a_item in array_a.borrow_all_items() {
                items.push(compute_binary_operation_impl(a_item, operator, b));
            }
            i::KnownData::Array(NVec::from_vec_and_dims(items, dimensions))
        }
    } else {
        if let i::KnownData::Array(array_b) = b {
            let dimensions = array_b.borrow_dimensions().clone();
            let mut items = Vec::with_capacity(array_b.borrow_all_items().len());
            for b_item in array_b.borrow_all_items() {
                items.push(compute_binary_operation_impl(a, operator, b_item));
            }
            i::KnownData::Array(NVec::from_vec_and_dims(items, dimensions))
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
            _ => unreachable!(),
        },
        i::BinaryOperator::IntDiv => match a {
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
    expr: Expression,
    from: &DataType,
    to: &DataType,
    program: &Program,
) -> Result<Expression, CompileProblem> {
    if from.equivalent(to, program) {
        return Result::Ok(expr);
    }
    // TODO: Nice error if array dimension not int.
    let from_dims: Vec<_> = from
        .borrow_dimensions()
        .iter()
        .map(|dim| program.borrow_value_of(dim).require_int())
        .collect();
    let to_dims: Vec<_> = to
        .borrow_dimensions()
        .iter()
        .map(|dim| program.borrow_value_of(dim).require_int())
        .collect();
    let mut final_dims = Vec::new();
    for index in 0..from_dims.len().max(to_dims.len()) {
        if index >= to_dims.len() {
            panic!("TODO: nice error, cannot inflate to smaller dimension array.");
        } else if index >= from_dims.len() {
            final_dims.push((to_dims[index], ProxyMode::Discard));
        } else if to_dims[index] == from_dims[index] {
            final_dims.push((to_dims[index], ProxyMode::Literal));
        } else if from_dims[index] == 1 {
            final_dims.push((to_dims[index], ProxyMode::Collapse));
        } else {
            panic!("TODO: nice error, invalid inflation.");
        }
    }
    Result::Ok(Expression::Proxy {
        base: Box::new(expr),
        dimensions: final_dims,
    })
}
