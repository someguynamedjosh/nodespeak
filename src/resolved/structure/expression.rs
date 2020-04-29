use crate::problem::FilePosition;
use crate::resolved::structure::{KnownData, ScopeId, VariableId};
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
pub enum VPExpression {
    Literal(KnownData, FilePosition),
    Variable(VariableId, FilePosition),
    Collect(Vec<VPExpression>, FilePosition),
    BuildArrayType {
        dimensions: Vec<VPExpression>,
        base: Box<VPExpression>,
        position: FilePosition,
    },

    UnaryOperation(UnaryOperator, Box<VPExpression>, FilePosition),
    BinaryOperation(
        Box<VPExpression>,
        BinaryOperator,
        Box<VPExpression>,
        FilePosition,
    ),
    Index {
        base: Box<VPExpression>,
        indexes: Vec<VPExpression>,
        position: FilePosition,
    },
    FuncCall {
        function: Box<VPExpression>,
        position: FilePosition,
    },
}

impl Debug for VPExpression {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        match self {
            Self::Literal(value, ..) => write!(formatter, "{:?}", value),
            Self::Variable(id, ..) => write!(formatter, "{:?}", id),
            Self::Collect(values, ..) => {
                write!(formatter, "[")?;
                for value in values.iter() {
                    write!(formatter, "{:?}, ", value)?;
                }
                write!(formatter, "]")
            }
            Self::BuildArrayType {
                dimensions, base, ..
            } => {
                for dimension in dimensions.iter() {
                    write!(formatter, "[{:?}]", dimension)?;
                }
                write!(formatter, "{:?}", base)
            }

            Self::UnaryOperation(operator, value, ..) => {
                write!(formatter, "({:?} {:?})", operator, value)
            }
            Self::BinaryOperation(v1, operator, v2, ..) => {
                write!(formatter, "({:?} {:?} {:?})", v1, operator, v2)
            }
            Self::Index { base, indexes, .. } => {
                write!(formatter, "{:?}", base)?;
                for index in indexes.iter() {
                    write!(formatter, "[{:?}]", index)?;
                }
                write!(formatter, "")
            }

            Self::FuncCall {
                function,
                ..
            } => {
                write!(formatter, "call {:?}", function)
            }
        }
    }
}

impl VPExpression {
    pub fn clone_position(&self) -> FilePosition {
        match self {
            Self::Literal(_, position)
            | Self::Variable(_, position)
            | Self::Collect(_, position)
            | Self::BuildArrayType { position, .. }
            | Self::UnaryOperation(_, _, position)
            | Self::BinaryOperation(_, _, _, position)
            | Self::Index { position, .. }
            | Self::FuncCall { position, .. } => position.clone(),
        }
    }
}

#[derive(Clone, PartialEq)]
pub enum VCExpression {
    Variable(VariableId, FilePosition),
    Index {
        base: Box<VCExpression>,
        indexes: Vec<VPExpression>,
        position: FilePosition,
    },
}

impl Debug for VCExpression {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        match self {
            Self::Variable(id, ..) => write!(formatter, "var {:?}", id),
            Self::Index { base, indexes, .. } => {
                write!(formatter, "({:?})", base)?;
                for index in indexes {
                    write!(formatter, "[{:?}]", index)?;
                }
                write!(formatter, "")
            }
        }
    }
}

impl VCExpression {
    pub fn clone_position(&self) -> FilePosition {
        match self {
            Self::Variable(_, position) | Self::Index { position, .. } => position.clone(),
        }
    }
}

#[derive(Clone, PartialEq)]
pub enum FuncCallOutput {
    InlineReturn(FilePosition),
    VCExpression(Box<VCExpression>),
}

impl Debug for FuncCallOutput {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        match self {
            Self::InlineReturn(..) => write!(formatter, "inline"),
            Self::VCExpression(expr) => write!(formatter, "{:?}", expr),
        }
    }
}

impl FuncCallOutput {
    pub fn clone_position(&self) -> FilePosition {
        match self {
            Self::InlineReturn(pos) => pos.clone(),
            Self::VCExpression(expr) => expr.clone_position(),
        }
    }
}

#[derive(Clone, PartialEq)]
pub enum Statement {
    CreationPoint {
        var: VariableId,
        var_type: Box<VPExpression>,
        position: FilePosition,
    },
    Assert(Box<VPExpression>, FilePosition),
    Return(FilePosition),
    Assign {
        target: Box<VCExpression>,
        value: Box<VPExpression>,
        position: FilePosition,
    },
    Branch {
        clauses: Vec<(VPExpression, ScopeId)>,
        else_clause: Option<ScopeId>,
        position: FilePosition,
    },
    ForLoop {
        counter: VariableId,
        start: Box<VPExpression>,
        end: Box<VPExpression>,
        body: ScopeId,
        position: FilePosition,
    },
    RawVPExpression(Box<VPExpression>),
}

impl Debug for Statement {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        match self {
            Self::CreationPoint { var, var_type, .. } => {
                write!(formatter, "define {:?} {:?}", var_type, var)
            }
            Self::Assert(value, ..) => write!(formatter, "assert {:?};", value),
            Self::Return(..) => write!(formatter, "return;"),
            Self::Assign { target, value, .. } => write!(formatter, "{:?} = {:?};", target, value),
            Self::Branch {
                clauses,
                else_clause,
                ..
            } => {
                assert!(clauses.len() > 0);
                write!(formatter, "if {:?} {{ {:?} }}", &clauses[0].0, clauses[0].1)?;
                for (condition, scope) in &clauses[1..] {
                    write!(formatter, " else if {:?} {{ {:?} }}", condition, scope)?;
                }
                if let Some(scope) = else_clause {
                    write!(formatter, " else {{ {:?} }}", scope)?;
                }
                write!(formatter, "")
            }
            Self::ForLoop {
                counter,
                start,
                end,
                body,
                ..
            } => write!(
                formatter,
                "for {:?} = {:?} to {:?} {{ {:?} }}",
                counter, start, end, body
            ),
            Self::RawVPExpression(expr) => write!(formatter, "{:?}", expr),
        }
    }
}

impl Statement {
    pub fn clone_position(&self) -> FilePosition {
        match self {
            Self::CreationPoint { position, .. }
            | Self::Assert(_, position)
            | Self::Return(position)
            | Self::Assign { position, .. }
            | Self::Branch { position, .. }
            | Self::ForLoop { position, .. } => position.clone(),
            Self::RawVPExpression(expr) => expr.clone_position(),
        }
    }
}
