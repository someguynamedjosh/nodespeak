pub use crate::trivial::structure::Variable;
pub use crate::shared::{NativeType, NativeBaseType};

mod program;
mod value;
mod variable;

pub use program::*;
pub use value::*;
pub use variable::*;

pub mod universal {
    pub use super::{LiteralData, Value, Variable, VariableId, NativeType, NativeBaseType};
}
