mod instructions;

use crate::specialized::general::GenericProgram;

pub use instructions::*;
pub use crate::specialized::general::universal::*;
pub type Program = GenericProgram<Instruction>;