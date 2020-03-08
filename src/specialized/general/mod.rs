pub use crate::shared::{NativeBaseType, NativeType, NativeVar, ProxyMode};

mod program;
mod value;

pub use program::*;
pub use value::*;

pub mod universal {
    pub use super::{
        Index, NativeBaseType, NativeType, NativeVar, ProxyMode, Value, VariableId,
    };
}
