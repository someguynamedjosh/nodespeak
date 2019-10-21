use crate::problem::FilePosition;
use crate::vague::structure::{BaseType, BinaryOperator, DataType, Expression, KnownData, Program};
use crate::util::NVec;
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
    a: &KnownData,
    operator: BinaryOperator,
    b: &KnownData,
) -> KnownData {
    if let KnownData::Array(array_a) = a {
        if let KnownData::Array(array_b) = b {
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
            KnownData::Array(NVec::from_vec_and_dims(result_items, ab_dims))
        } else {
            let dimensions = array_a.borrow_dimensions().clone();
            let mut items = Vec::with_capacity(array_a.borrow_all_items().len());
            for a_item in array_a.borrow_all_items() {
                items.push(compute_binary_operation_impl(a_item, operator, b));
            }
            KnownData::Array(NVec::from_vec_and_dims(items, dimensions))
        }
    } else {
        if let KnownData::Array(array_b) = b {
            let dimensions = array_b.borrow_dimensions().clone();
            let mut items = Vec::with_capacity(array_b.borrow_all_items().len());
            for b_item in array_b.borrow_all_items() {
                items.push(compute_binary_operation_impl(a, operator, b_item));
            }
            KnownData::Array(NVec::from_vec_and_dims(items, dimensions))
        } else {
            compute_binary_operation_impl(a, operator, b)
        }
    }
}

fn compute_binary_operation_impl(
    a: &KnownData,
    operator: BinaryOperator,
    b: &KnownData,
) -> KnownData {
    match operator {
        BinaryOperator::Add => match a {
            KnownData::Bool(..) => unimplemented!(),
            KnownData::Int(value) => KnownData::Int(value + b.require_int()),
            KnownData::Float(value) => KnownData::Float(value + b.require_float()),
            _ => unreachable!(),
        },
        BinaryOperator::Subtract => match a {
            KnownData::Bool(..) => unimplemented!(),
            KnownData::Int(value) => KnownData::Int(value - b.require_int()),
            KnownData::Float(value) => KnownData::Float(value - b.require_float()),
            _ => unreachable!(),
        },
        BinaryOperator::Multiply => match a {
            KnownData::Bool(..) => unimplemented!(),
            KnownData::Int(value) => KnownData::Int(value * b.require_int()),
            KnownData::Float(value) => KnownData::Float(value * b.require_float()),
            _ => unreachable!(),
        },
        BinaryOperator::Divide => match a {
            KnownData::Float(value) => KnownData::Float(value / b.require_float()),
            _ => unreachable!(),
        },
        BinaryOperator::IntDiv => match a {
            KnownData::Int(value) => KnownData::Int(value / b.require_int()),
            _ => unreachable!(),
        },
        BinaryOperator::Modulo => match a {
            KnownData::Bool(..) => unimplemented!(),
            KnownData::Int(value) => KnownData::Int(value % b.require_int()),
            KnownData::Float(value) => KnownData::Float(value % b.require_float()),
            _ => unreachable!(),
        },
        BinaryOperator::Power => match a {
            KnownData::Bool(..) => unimplemented!(),
            KnownData::Int(value) => {
                KnownData::Int(i64::pow(*value, b.require_int().try_into().unwrap()))
            }
            KnownData::Float(value) => KnownData::Float(value.powf(b.require_float())),
            _ => unreachable!(),
        },
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
            _ => unreachable!(),
        },
        BinaryOperator::NotEqual => match a {
            KnownData::Bool(value) => KnownData::Bool(*value != b.require_bool()),
            KnownData::Int(value) => KnownData::Bool(*value != b.require_int()),
            KnownData::Float(value) => KnownData::Bool(*value != b.require_float()),
            KnownData::DataType(value) => KnownData::Bool(value != b.require_data_type()),
            KnownData::Function(value) => KnownData::Bool(value != b.require_function()),
            KnownData::Array(value) => KnownData::Bool(value != b.require_array()),
            _ => unreachable!(),
        },
        BinaryOperator::LessThan => match a {
            KnownData::Int(value) => KnownData::Bool(*value < b.require_int()),
            KnownData::Float(value) => KnownData::Bool(*value < b.require_float()),
            _ => unreachable!(),
        },
        BinaryOperator::GreaterThan => match a {
            KnownData::Int(value) => KnownData::Bool(*value > b.require_int()),
            KnownData::Float(value) => KnownData::Bool(*value > b.require_float()),
            _ => unreachable!(),
        },
        BinaryOperator::LessThanOrEqual => match a {
            KnownData::Int(value) => KnownData::Bool(*value <= b.require_int()),
            KnownData::Float(value) => KnownData::Bool(*value <= b.require_float()),
            _ => unreachable!(),
        },
        BinaryOperator::GreaterThanOrEqual => match a {
            KnownData::Int(value) => KnownData::Bool(*value >= b.require_int()),
            KnownData::Float(value) => KnownData::Bool(*value >= b.require_float()),
            _ => unreachable!(),
        },
    }
}

fn biggest_scalar(a: &BaseType, b: &BaseType, program: &Program) -> Result<BaseType, ()> {
    // BCT rule 1
    if a == &BaseType::Automatic {
        Result::Ok(b.clone())
    } else if b == &BaseType::Automatic {
        Result::Ok(a.clone())
    } else if a.equivalent(b, program) {
        Result::Ok(a.clone())
    } else {
        Result::Err(())
    }
}

/// Returns Result::Err if there is no biggest type.
pub(super) fn biggest_type(a: &DataType, b: &DataType, program: &Program) -> Result<DataType, ()> {
    // BCT rule 1
    if a.is_specific_scalar(&BaseType::Automatic) {
        Result::Ok(b.clone())
    } else if b.is_specific_scalar(&BaseType::Automatic) {
        Result::Ok(a.clone())
    // BCT rule 2
    } else if a.equivalent(b, program) {
        Result::Ok(a.clone())
    // BCT rules 3 - 5
    } else if a.is_array() || b.is_array() {
        // TODO: Nice error if array dimension not int.
        let a_dims: Vec<_> = a
            .borrow_dimensions()
            .iter()
            .map(|dim| program.borrow_value_of(dim).require_int())
            .collect();
        let b_dims: Vec<_> = a
            .borrow_dimensions()
            .iter()
            .map(|dim| program.borrow_value_of(dim).require_int())
            .collect();
        let mut final_dims = Vec::new();
        for index in 0..a_dims.len().max(b_dims.len()) {
            // BCT rule 3
            if a_dims[index] == b_dims[index] {
                final_dims.push(a_dims[index]);
            // BCT rule 4
            } else if a_dims[index] == 1 {
                final_dims.push(b_dims[index]);
            } else if b_dims[index] == 1 {
                final_dims.push(a_dims[index]);
            // BCT rule 5
            } else if index >= a_dims.len() {
                final_dims.push(b_dims[index]);
            } else if index >= b_dims.len() {
                final_dims.push(a_dims[index]);
            } else {
                return Result::Err(());
            }
        }
        let base_type = biggest_scalar(a.borrow_base(), b.borrow_base(), program)?;
        Result::Ok(DataType::array(
            base_type,
            final_dims
                .into_iter()
                .map(|dim| Expression::Literal(KnownData::Int(dim), FilePosition::placeholder()))
                .collect(),
        ))
    } else {
        Result::Err(())
    }
}
