use crate::values::{ValuePtr, LocalPtr};

#[derive(Clone, Debug, PartialEq)]
pub enum Type {
    Float,
    Int,
    Bool,
    Array {
        eltype: ValuePtr,
        dims: Vec<ValuePtr>,
    },
    InSet{
        base_type: ValuePtr,
        allowed_values: Vec<ValuePtr>
    },
    Function {
        inputs: Vec<LocalPtr>,
        outputs: Vec<LocalPtr>,
    },
    Malformed,
}
