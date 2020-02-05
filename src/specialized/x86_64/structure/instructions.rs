use crate::specialized::general::Value;

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

pub enum ParallelWidth {
    W128,
    W256,
    W512,
}

impl Debug for ParallelWidth {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        match self {
            Self::W128 => write!(formatter, "128"),
            Self::W256 => write!(formatter, "256"),
            Self::W512 => write!(formatter, "512"),
        }
    }
}

pub enum BinaryOperator {
    AddI,
    SubI,
    MulI,
    DivI,
    ModI,

    AddPackedI(ParallelWidth),
    SubPackedI(ParallelWidth),
    MulPackedI(ParallelWidth),
    DivPackedI(ParallelWidth),
    ModPackedI(ParallelWidth),

    AddF,
    SubF,
    MulF,
    DivF,
    ModF,

    AddPackedF(ParallelWidth),
    SubPackedF(ParallelWidth),
    MulPackedF(ParallelWidth),
    DivPackedF(ParallelWidth),
    ModPackedF(ParallelWidth),

    BAnd,
    BOr,
    BXor,
}

impl Debug for BinaryOperator {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        match self {
            Self::AddI => write!(formatter, "addi"),
            Self::SubI => write!(formatter, "subi"),
            Self::MulI => write!(formatter, "muli"),
            Self::DivI => write!(formatter, "divi"),
            Self::ModI => write!(formatter, "modi"),

            Self::AddPackedI(width) => write!(formatter, "addpackedi{:?}", width),
            Self::SubPackedI(width) => write!(formatter, "subpackedi{:?}", width),
            Self::MulPackedI(width) => write!(formatter, "mulpackedi{:?}", width),
            Self::DivPackedI(width) => write!(formatter, "divpackedi{:?}", width),
            Self::ModPackedI(width) => write!(formatter, "modpackedi{:?}", width),

            Self::AddF => write!(formatter, "addf"),
            Self::SubF => write!(formatter, "subf"),
            Self::MulF => write!(formatter, "mulf"),
            Self::DivF => write!(formatter, "divf"),
            Self::ModF => write!(formatter, "modf"),

            Self::AddPackedF(width) => write!(formatter, "addpackedf{:?}", width),
            Self::SubPackedF(width) => write!(formatter, "subpackedf{:?}", width),
            Self::MulPackedF(width) => write!(formatter, "mulpackedf{:?}", width),
            Self::DivPackedF(width) => write!(formatter, "divpackedf{:?}", width),
            Self::ModPackedF(width) => write!(formatter, "modpackedf{:?}", width),

            Self::BAnd => write!(formatter, "band"),
            Self::BOr => write!(formatter, "bor"),
            Self::BXor => write!(formatter, "bxor"),
        }
    }
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

    Compare {
        a: Value,
        b: Value,
    },
    Assert(Condition),
}

impl Debug for Instruction {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        match self {
            Instruction::Move { from, to } => write!(formatter, "move {:?} -> {:?}", from, to),

            Instruction::BinaryOperation { op, a, b, x } => {
                write!(formatter, "{:?} {:?}, {:?} -> {:?}", op, a, b, x)
            }

            Instruction::Compare { a, b } => write!(formatter, "comp {:?}, {:?}", a, b),

            Instruction::Assert(value) => write!(formatter, "asrt {:?}", value),
        }
    }
}

impl crate::specialized::general::Instruction for Instruction {
    fn reads_from(&self) -> Vec<&Value> {
        match self {
            Self::Move { from, .. } => vec![from],
            Self::BinaryOperation { a, b, .. } => vec![a, b],
            Self::Compare { a, b } => vec![a, b],
            Self::Assert(..) => vec![],
        }
    }

    fn writes_to(&self) -> Vec<&Value> {
        match self {
            Self::Move { to, .. } => vec![to],
            Self::BinaryOperation { x, .. } => vec![x],
            Self::Compare { .. } | Self::Assert(..) => vec![],
        }
    }
}
