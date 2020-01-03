pub use crate::trivial::structure::{Variable, VariableType};

mod program;
mod value;
mod variable;

pub use program::*;
pub use value::*;
pub use variable::*;

pub mod universal {
    pub use super::{Value, Variable, VariableId, VariableType};
}
