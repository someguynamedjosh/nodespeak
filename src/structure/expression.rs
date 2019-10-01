use crate::problem::FilePosition;
use crate::structure::{KnownData, VariableId};
use std::fmt::{self, Debug, Formatter};

#[derive(Clone, Copy, PartialEq)]
pub enum UnaryOperator {
    Negate,
    Not,
    BNot,
    Reciprocal,
}

impl Debug for UnaryOperator {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        match self {
            UnaryOperator::Negate => write!(formatter, "-"),
            UnaryOperator::Not => write!(formatter, "not"),
            UnaryOperator::BNot => write!(formatter, "bnot"),
            UnaryOperator::Reciprocal => write!(formatter, "reciprocal of"),
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
pub enum BinaryOperator {
    Add,
    Subtract,
    Multiply,
    Divide,
    IntDiv,
    Modulo,
    Power,

    And,
    Or,
    Xor,
    BAnd,
    BOr,
    BXor,

    Equal,
    NotEqual,
    LessThan,
    GreaterThan,
    LessThanOrEqual,
    GreaterThanOrEqual,
}

impl Debug for BinaryOperator {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        match self {
            BinaryOperator::Add => write!(formatter, "+"),
            BinaryOperator::Subtract => write!(formatter, "-"),
            BinaryOperator::Multiply => write!(formatter, "*"),
            BinaryOperator::Divide => write!(formatter, "/"),
            BinaryOperator::IntDiv => write!(formatter, "//"),
            BinaryOperator::Modulo => write!(formatter, "%"),
            BinaryOperator::Power => write!(formatter, "**"),

            BinaryOperator::And => write!(formatter, "and"),
            BinaryOperator::Or => write!(formatter, "or"),
            BinaryOperator::Xor => write!(formatter, "xor"),
            BinaryOperator::BAnd => write!(formatter, "band"),
            BinaryOperator::BOr => write!(formatter, "bor"),
            BinaryOperator::BXor => write!(formatter, "bxor"),

            BinaryOperator::Equal => write!(formatter, "=="),
            BinaryOperator::NotEqual => write!(formatter, "!="),
            BinaryOperator::LessThan => write!(formatter, "<"),
            BinaryOperator::GreaterThan => write!(formatter, ">"),
            BinaryOperator::LessThanOrEqual => write!(formatter, "<="),
            BinaryOperator::GreaterThanOrEqual => write!(formatter, ">="),
        }
    }
}

#[derive(Clone, PartialEq)]
pub enum Expression {
    Literal(KnownData, FilePosition),
    Variable(VariableId, FilePosition),
    Access {
        base: Box<Expression>,
        indexes: Vec<Expression>,
        position: FilePosition,
    },
    InlineReturn(FilePosition),

    UnaryOperation(UnaryOperator, Box<Expression>, FilePosition),
    BinaryOperation(
        Box<Expression>,
        BinaryOperator,
        Box<Expression>,
        FilePosition,
    ),

    Collect(Vec<Expression>, FilePosition),
    CreationPoint(VariableId, FilePosition),

    Assert(Box<Expression>, FilePosition),
    Assign {
        target: Box<Expression>,
        value: Box<Expression>,
        position: FilePosition,
    },
    Return(FilePosition),

    FuncCall {
        function: Box<Expression>,
        inputs: Vec<Expression>,
        outputs: Vec<Expression>,
        position: FilePosition,
    },
}

impl Debug for Expression {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        match self {
            Expression::Literal(value, ..) => write!(formatter, "{:?}", value),
            Expression::Variable(id, ..) => write!(formatter, "{:?}", id),
            Expression::Access { base, indexes, .. } => {
                write!(formatter, "{:?}", base)?;
                for index in indexes.iter() {
                    write!(formatter, "[{:?}]", index)?;
                }
                write!(formatter, "")
            }
            Expression::InlineReturn(..) => write!(formatter, "inline"),

            Expression::UnaryOperation(operator, value, ..) => {
                write!(formatter, "({:?} {:?})", operator, value)
            }
            Expression::BinaryOperation(v1, operator, v2, ..) => {
                write!(formatter, "({:?} {:?} {:?})", v1, operator, v2)
            }

            Expression::Collect(values, ..) => {
                write!(formatter, "[")?;
                for value in values.iter() {
                    write!(formatter, "{:?}, ", value)?;
                }
                write!(formatter, "]")
            }
            Expression::CreationPoint(var_id, ..) => write!(formatter, "define {:?}", var_id),

            Expression::Assert(value, ..) => write!(formatter, "assert {:?};", value),
            Expression::Assign { target, value, .. } => {
                write!(formatter, "{:?} = {:?};", target, value)
            }
            Expression::Return(..) => write!(formatter, "return;"),

            Expression::FuncCall {
                function,
                inputs,
                outputs,
                ..
            } => {
                write!(formatter, "{:?}(", function)?;
                for input in inputs {
                    write!(formatter, "{:?}, ", input)?;
                }
                write!(formatter, "):(")?;
                for output in outputs {
                    write!(formatter, "{:?}, ", output)?;
                }
                write!(formatter, ")")
            }
        }
    }
}

impl Expression {
    pub fn clone_position(&self) -> FilePosition {
        match self {
            Expression::Literal(_, position)
            | Expression::Variable(_, position)
            | Expression::Access { position, .. }
            | Expression::InlineReturn(position)
            | Expression::UnaryOperation(_, _, position)
            | Expression::BinaryOperation(_, _, _, position)
            | Expression::Collect(_, position)
            | Expression::CreationPoint(_, position)
            | Expression::Assert(_, position)
            | Expression::Assign { position, .. }
            | Expression::Return(position)
            | Expression::FuncCall { position, .. } => position.clone(),
        }
    }
}
