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
    InRange(ValuePtr, ValuePtr),
    InSet(Vec<ValuePtr>),
    Function {
        inputs: Vec<ParameterPtr>,
        outputs: Vec<ParameterPtr>,
    },
    Never,
}
