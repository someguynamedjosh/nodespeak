mod instructions;

use crate::specialized::general::GenericProgram;

pub use crate::specialized::general::universal::*;
pub use instructions::*;
pub type Program = GenericProgram<Instruction>;
pub type AnnotatedInstruction = crate::specialized::general::WrappedInstruction<Instruction>;
