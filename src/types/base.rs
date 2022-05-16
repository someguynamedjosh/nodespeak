use crate::values::{ValuePtr, ParameterPtr};

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
        inputs: Vec<ParameterPtr>,
        outputs: Vec<ParameterPtr>,
    },
    Malformed,
}
