use super::{LabelId, Value};

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
    CompI(Condition),
    CompF(Condition),
}

pub enum Instruction {
    Move {
        from: Value,
        to: Value,
    },
    Load {
        from: Value,
        from_indexes: Vec<Value>,
        to: Value,
    },
    Store {
        from: Value,
        to: Value,
        to_indexes: Vec<Value>,
    },

    BinaryOperation {
        op: BinaryOperator,
        a: Value,
        b: Value,
        x: Value,
    },

    Label(LabelId),
    Jump {
        label: LabelId,
    },
    Branch {
        condition: Value,
        true_target: LabelId,
        false_target: LabelId,
    },
}

impl Debug for Instruction {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        match self {
            Instruction::Move { from, to } => write!(formatter, "move {:?} -> {:?}", from, to),
            Instruction::Load {
                from,
                to,
                from_indexes,
            } => {
                write!(formatter, "load {:?}", from)?;
                for index in from_indexes {
                    write!(formatter, "[{:?}]", index)?;
                }
                write!(formatter, " -> {:?}", to)
            },
            Instruction::Store {
                from,
                to,
                to_indexes,
            } => {
                write!(formatter, "store {:?} -> {:?}", from, to)?;
                for index in to_indexes {
                    write!(formatter, "[{:?}]", index)?;
                }
                write!(formatter, "")
            },

            Instruction::BinaryOperation { op, a, b, x } => write!(
                formatter,
                "{} {:?}, {:?} -> {:?}",
                match op {
                    BinaryOperator::AddI => "addi".to_owned(),
                    BinaryOperator::SubI => "subi".to_owned(),
                    BinaryOperator::MulI => "muli".to_owned(),
                    BinaryOperator::DivI => "divi".to_owned(),
                    BinaryOperator::ModI => "modi".to_owned(),

                    BinaryOperator::AddF => "addf".to_owned(),
                    BinaryOperator::SubF => "subf".to_owned(),
                    BinaryOperator::MulF => "mulf".to_owned(),
                    BinaryOperator::DivF => "divf".to_owned(),
                    BinaryOperator::ModF => "modf".to_owned(),

                    BinaryOperator::BAnd => "band".to_owned(),
                    BinaryOperator::BOr => "bor ".to_owned(),
                    BinaryOperator::BXor => "bxor".to_owned(),

                    BinaryOperator::CompI(cond) => format!("compi {:?}", cond),
                    BinaryOperator::CompF(cond) => format!("compf {:?}", cond),
                },
                a,
                b,
                x
            ),

            Instruction::Label(id) => write!(formatter, "labl {:?}", id),
            Instruction::Jump { label } => write!(formatter, "jump to {:?}", label),
            Instruction::Branch {
                condition,
                true_target,
                false_target,
            } => write!(
                formatter,
                "if {:?} jump to {:?} else {:?}",
                condition, true_target, false_target
            ),
        }
    }
}
