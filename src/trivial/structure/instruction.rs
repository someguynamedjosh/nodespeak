use crate::trivial::structure::{LabelId, VariableId};

use std::fmt::{self, Debug, Formatter};

#[derive(Clone, Copy, Debug)]
pub enum Condition {
    LessThan,
    GreaterThan,
    LessThanOrEqual,
    GreaterThanOrEqual,
    Equal,
    NotEqual,
    Unconditional,
}

pub enum Instruction {
    Move {
        from: VariableId,
        to: VariableId,
    },

    AddI {
        a: VariableId,
        b: VariableId,
        x: VariableId,
    },
    SubI {
        a: VariableId,
        b: VariableId,
        x: VariableId,
    },
    MulI {
        a: VariableId,
        b: VariableId,
        x: VariableId,
    },
    DivI {
        a: VariableId,
        b: VariableId,
        x: VariableId,
    },

    AddF {
        a: VariableId,
        b: VariableId,
        x: VariableId,
    },
    SubF {
        a: VariableId,
        b: VariableId,
        x: VariableId,
    },
    MulF {
        a: VariableId,
        b: VariableId,
        x: VariableId,
    },
    DivF {
        a: VariableId,
        b: VariableId,
        x: VariableId,
    },

    And {
        a: VariableId,
        b: VariableId,
        x: VariableId,
    },
    Or {
        a: VariableId,
        b: VariableId,
        x: VariableId,
    },
    Xor {
        a: VariableId,
        b: VariableId,
        x: VariableId,
    },
    Not {
        a: VariableId,
        x: VariableId,
    },

    Compare {
        a: VariableId,
        b: VariableId,
    },
    Label(LabelId),
    Jump {
        label: LabelId,
        condition: Condition,
    },
}

impl Debug for Instruction {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        match self {
            Instruction::Move { from, to } => writeln!(formatter, "move {:?} -> {:?}", from, to),

            Instruction::AddI { a, b, x } => {
                writeln!(formatter, "addi {:?}, {:?} -> {:?}", a, b, x)
            }
            Instruction::SubI { a, b, x } => {
                writeln!(formatter, "subi {:?}, {:?} -> {:?}", a, b, x)
            }
            Instruction::MulI { a, b, x } => {
                writeln!(formatter, "muli {:?}, {:?} -> {:?}", a, b, x)
            }
            Instruction::DivI { a, b, x } => {
                writeln!(formatter, "divi {:?}, {:?} -> {:?}", a, b, x)
            }

            Instruction::AddF { a, b, x } => {
                writeln!(formatter, "addf {:?}, {:?} -> {:?}", a, b, x)
            }
            Instruction::SubF { a, b, x } => {
                writeln!(formatter, "subf {:?}, {:?} -> {:?}", a, b, x)
            }
            Instruction::MulF { a, b, x } => {
                writeln!(formatter, "mulf {:?}, {:?} -> {:?}", a, b, x)
            }
            Instruction::DivF { a, b, x } => {
                writeln!(formatter, "divf {:?}, {:?} -> {:?}", a, b, x)
            }

            Instruction::And { a, b, x } => writeln!(formatter, "and  {:?}, {:?} -> {:?}", a, b, x),
            Instruction::Or { a, b, x } => writeln!(formatter, "or   {:?}, {:?} -> {:?}", a, b, x),
            Instruction::Xor { a, b, x } => writeln!(formatter, "xor  {:?}, {:?} -> {:?}", a, b, x),
            Instruction::Not { a, x } => writeln!(formatter, "not  {:?}, -> {:?}", a, x),

            Instruction::Compare { a, b } => writeln!(formatter, "comp {:?}, {:?}", a, b),
            Instruction::Label(id) => writeln!(formatter, "labl {:?}", id),
            Instruction::Jump { label, condition } => {
                writeln!(formatter, "jump {:?} if {:?}", label, condition)
            }
        }
    }
}
