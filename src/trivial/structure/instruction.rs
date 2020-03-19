use crate::shared::LabelId;
use crate::trivial::structure::Value;

use std::fmt::{self, Debug, Formatter};

#[derive(Clone, Copy, Debug)]
pub enum Condition {
    LessThan,
    GreaterThan,
    LessThanOrEqual,
    GreaterThanOrEqual,
    Equal,
    NotEqual,
}

impl Condition {
    pub fn as_negative(&self) -> Condition {
        match self {
            Self::LessThan => Self::GreaterThanOrEqual,
            Self::GreaterThan => Self::LessThanOrEqual,
            Self::LessThanOrEqual => Self::GreaterThan,
            Self::GreaterThanOrEqual => Self::LessThan,
            Self::Equal => Self::NotEqual,
            Self::NotEqual => Self::Equal,
        }
    }
}

pub enum BinaryOperator {
    AddI,
    SubI,
    MulI,
    DivI,
    ModI,

    AddF,
    SubF,
    MulF,
    DivF,
    ModF,

    BAnd,
    BOr,
    BXor,
}

pub enum Instruction {
    Move {
        from: Value,
        to: Value,
    },

    BinaryOperation {
        op: BinaryOperator,
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
    },
    ConditionalJump {
        label: LabelId,
        condition: Condition,
    },

    Assert(Condition),
}

impl Debug for Instruction {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        match self {
            Instruction::Move { from, to } => write!(formatter, "move {:?} -> {:?}", from, to),

            Instruction::BinaryOperation { op, a, b, x } => write!(
                formatter,
                "{} {:?}, {:?} -> {:?}",
                match op {
                    BinaryOperator::AddI => "addi",
                    BinaryOperator::SubI => "subi",
                    BinaryOperator::MulI => "muli",
                    BinaryOperator::DivI => "divi",
                    BinaryOperator::ModI => "modi",

                    BinaryOperator::AddF => "addf",
                    BinaryOperator::SubF => "subf",
                    BinaryOperator::MulF => "mulf",
                    BinaryOperator::DivF => "divf",
                    BinaryOperator::ModF => "modf",

                    BinaryOperator::BAnd => "band",
                    BinaryOperator::BOr => "bor ",
                    BinaryOperator::BXor => "bxor",
                },
                a,
                b,
                x
            ),
            Instruction::Not { a, x } => write!(formatter, "not  {:?}, -> {:?}", a, x),

            Instruction::Compare { a, b } => write!(formatter, "comp {:?}, {:?}", a, b),
            Instruction::Label(id) => write!(formatter, "labl {:?}", id),
            Instruction::Jump { label } => write!(formatter, "jump to {:?}", label),
            Instruction::ConditionalJump { label, condition } => {
                write!(formatter, "jump to {:?} if {:?}", label, condition)
            }

            Instruction::Assert(value) => write!(formatter, "asrt {:?}", value),
        }
    }
}
