use crate::values::{simplify::SimplificationContext, LocalPtr, ValuePtr};

#[derive(Clone, Debug, PartialEq)]
pub enum Type {
    Type,
    Float,
    Int,
    Bool,
    Array {
        eltype: ValuePtr,
        dims: Vec<ValuePtr>,
    },
    InSet {
        base_type: ValuePtr,
        allowed_values: Vec<ValuePtr>,
    },
    Function {
        inputs: Vec<LocalPtr>,
        outputs: Vec<LocalPtr>,
    },
    Malformed,
}

impl Type {
    pub(crate) fn simplify(&self, ctx: &mut SimplificationContext) -> Type {
        match self {
            Type::Type
            | Type::Float
            | Type::Int
            | Type::Bool
            | Type::Function { .. }
            | Type::Malformed => self.clone(),
            Type::Array { eltype, dims } => Type::Array {
                eltype: eltype.simplify(ctx),
                dims: dims.iter().map(|x| x.simplify(ctx)).collect(),
            },
            Type::InSet {
                base_type,
                allowed_values,
            } => Type::InSet {
                base_type: base_type.simplify(ctx),
                allowed_values: allowed_values.iter().map(|x| x.simplify(ctx)).collect(),
            },
        }
    }
}
