use super::{BuiltinType, Value, ValuePtr};

pub fn type_a_is_compatible_with_type_b(type_a: &ValuePtr, type_b: &ValuePtr) -> bool {
    match (&*type_a.borrow(), &*type_b.borrow()) {
        (Value::BuiltinType(..), Value::BuiltinType(BuiltinType::Type)) => true,
        (_, Value::BuiltinType(BuiltinType::Any)) => true,
        (Value::BuiltinType(BuiltinType::Bool), Value::BuiltinType(BuiltinType::Int)) => true,
        (Value::BuiltinType(BuiltinType::Bool), Value::BuiltinType(BuiltinType::Float)) => true,
        (Value::BuiltinType(BuiltinType::Int), Value::BuiltinType(BuiltinType::Float)) => true,
        (
            Value::BuiltinType(BuiltinType::InSet {
                eltype: a_eltype,
                elements: a_elements,
            }),
            Value::BuiltinType(BuiltinType::InSet {
                eltype: b_eltype,
                elements: b_elements,
            }),
        ) => {
            if !type_a_is_compatible_with_type_b(a_eltype, b_eltype) {
                return false;
            }
            // For every element in A, there must be a matching element in B.
            'a: for a in a_elements {
                for b in b_elements {
                    if a == b {
                        continue 'a;
                    }
                }
                return false;
            }
            true
        }
        (Value::BuiltinType(BuiltinType::InSet { eltype, .. }), _) => {
            type_a_is_compatible_with_type_b(eltype, type_b)
        }
        _ => type_a == type_b,
    }
}
