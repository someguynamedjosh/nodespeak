pub use crate::shared::NativeVar;
pub use crate::shared::{NativeType, NativeBaseType};

mod program;
mod value;

pub use program::*;
pub use value::*;

pub mod universal {
    pub use super::{LiteralData, Value, NativeVar, VariableId, NativeType, NativeBaseType};
}
