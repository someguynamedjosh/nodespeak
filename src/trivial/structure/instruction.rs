use crate::trivial::structure::{LabelId, VariableId, Value};

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
        from: Value,
        to: Value,
    },

    AddI {
        a: Value,
        b: Value,
        x: Value,
    },
    SubI {
        a: Value,
        b: Value,
        x: Value,
    },
    MulI {
        a: Value,
        b: Value,
        x: Value,
    },
    DivI {
        a: Value,
        b: Value,
        x: Value,
    },

    AddF {
        a: Value,
        b: Value,
        x: Value,
    },
    SubF {
        a: Value,
        b: Value,
        x: Value,
    },
    MulF {
        a: Value,
        b: Value,
        x: Value,
    },
    DivF {
        a: Value,
        b: Value,
        x: Value,
    },

    And {
        a: Value,
        b: Value,
        x: Value,
    },
    Or {
        a: Value,
        b: Value,
        x: Value,
    },
    Xor {
        a: Value,
        b: Value,
        x: Value,
    },
    Not {
        a: Value,
        x: Value,
    },

    Compare {
        a: Value,
        b: Value,
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
            Instruction::Move { from, to } => write!(formatter, "move {:?} -> {:?}", from, to),

            Instruction::AddI { a, b, x } => {
                write!(formatter, "addi {:?}, {:?} -> {:?}", a, b, x)
            }
            Instruction::SubI { a, b, x } => {
                write!(formatter, "subi {:?}, {:?} -> {:?}", a, b, x)
            }
            Instruction::MulI { a, b, x } => {
                write!(formatter, "muli {:?}, {:?} -> {:?}", a, b, x)
            }
            Instruction::DivI { a, b, x } => {
                write!(formatter, "divi {:?}, {:?} -> {:?}", a, b, x)
            }

            Instruction::AddF { a, b, x } => {
                write!(formatter, "addf {:?}, {:?} -> {:?}", a, b, x)
            }
            Instruction::SubF { a, b, x } => {
                write!(formatter, "subf {:?}, {:?} -> {:?}", a, b, x)
            }
            Instruction::MulF { a, b, x } => {
                write!(formatter, "mulf {:?}, {:?} -> {:?}", a, b, x)
            }
            Instruction::DivF { a, b, x } => {
                write!(formatter, "divf {:?}, {:?} -> {:?}", a, b, x)
            }

            Instruction::And { a, b, x } => write!(formatter, "and  {:?}, {:?} -> {:?}", a, b, x),
            Instruction::Or { a, b, x } => write!(formatter, "or   {:?}, {:?} -> {:?}", a, b, x),
            Instruction::Xor { a, b, x } => write!(formatter, "xor  {:?}, {:?} -> {:?}", a, b, x),
            Instruction::Not { a, x } => write!(formatter, "not  {:?}, -> {:?}", a, x),

            Instruction::Compare { a, b } => write!(formatter, "comp {:?}, {:?}", a, b),
            Instruction::Label(id) => write!(formatter, "labl {:?}", id),
            Instruction::Jump { label, condition } => {
                write!(formatter, "jump {:?} if {:?}", label, condition)
            }
        }
    }
}
