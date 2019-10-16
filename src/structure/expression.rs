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
    Proxy {
        base: Box<Expression>,
    },
    Access {
        base: Box<Expression>,
        indexes: Vec<Expression>,
        position: FilePosition,
    },
    InlineReturn(Box<Expression>, FilePosition),

    UnaryOperation(UnaryOperator, Box<Expression>, FilePosition),
    BinaryOperation(
        Box<Expression>,
        BinaryOperator,
        Box<Expression>,
        FilePosition,
    ),

    PickInput(VariableId, usize, FilePosition),
    PickOutput(VariableId, usize, FilePosition),
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
        setup: Vec<Expression>,
        teardown: Vec<Expression>,
        position: FilePosition,
    },
}

impl Debug for Expression {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        match self {
            Expression::Literal(value, ..) => write!(formatter, "{:?}", value),
            Expression::Variable(id, ..) => write!(formatter, "{:?}", id),
            Expression::Proxy { .. } => unimplemented!(),
            Expression::Access { base, indexes, .. } => {
                write!(formatter, "{:?}", base)?;
                for index in indexes.iter() {
                    write!(formatter, "[{:?}]", index)?;
                }
                write!(formatter, "")
            }
            Expression::InlineReturn(value, ..) => write!(formatter, "inline({:?})", value),

            Expression::UnaryOperation(operator, value, ..) => {
                write!(formatter, "({:?} {:?})", operator, value)
            }
            Expression::BinaryOperation(v1, operator, v2, ..) => {
                write!(formatter, "({:?} {:?} {:?})", v1, operator, v2)
            }

            Expression::PickInput(function, index, ..) => {
                write!(formatter, "input {} of {:?}", index, function)
            }
            Expression::PickOutput(function, index, ..) => {
                write!(formatter, "output {} of {:?}", index, function)
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
                setup,
                teardown,
                ..
            } => {
                write!(formatter, "{{ ")?;
                for expr in setup {
                    write!(formatter, "{:?} ", expr)?;
                }
                write!(formatter, "}} call {:?} {{ ", function)?;
                for expr in teardown {
                    write!(formatter, "{:?} ", expr)?;
                }
                write!(formatter, "}}")
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
            | Expression::InlineReturn(_, position)
            | Expression::UnaryOperation(_, _, position)
            | Expression::BinaryOperation(_, _, _, position)
            | Expression::PickInput(_, _, position)
            | Expression::PickOutput(_, _, position)
            | Expression::Collect(_, position)
            | Expression::CreationPoint(_, position)
            | Expression::Assert(_, position)
            | Expression::Assign { position, .. }
            | Expression::Return(position)
            | Expression::FuncCall { position, .. } => position.clone(),
            Expression::Proxy { base, .. } => base.clone_position(),
        }
    }

    /// Different expressions have some rules regarding what can or cannot be put in them. This
    /// function checks that all those rules are true:
    /// For a Literal, the data cannot be unknown.
    /// For an Access, the base must be a Literal, Variable, or Proxy of a Literal or Variable.
    /// For an Assign, the target must be a Variable or Proxy of a Variable.
    /// These rules will be performed recursively, so any expression that contains other expressions
    /// inside it will only be valid if all its child expressions are also valid. Note that this
    /// cannot check that child expressions will return the proper type to be part of the overall
    /// expression. For example, if you have an add expression with a Float on one side and an Int
    /// on the other, it does not violate any of the above rules, so it is 'valid'.
    pub fn is_valid(&self) -> bool {
        match self {
            Expression::Literal(value, ..) => value != &KnownData::Unknown,
            Expression::Variable(..) => true,
            Expression::Proxy { base, .. } => base.is_valid(),
            Expression::Access { base, indexes, .. } => {
                (match &**base {
                    Expression::Literal(..) | Expression::Variable(..) => base.is_valid(),
                    Expression::Proxy {
                        base: proxy_base, ..
                    } => match &**proxy_base {
                        Expression::Literal(..) | Expression::Variable(..) => base.is_valid(),
                        _ => false,
                    },
                    _ => false,
                }) && indexes
                    .iter()
                    .fold(true, |valid, index| valid && index.is_valid())
            }
            Expression::InlineReturn(..) => true,
            Expression::UnaryOperation(_, operand, ..) => operand.is_valid(),
            Expression::BinaryOperation(op1, _, op2, ..) => op1.is_valid() && op2.is_valid(),
            Expression::PickInput(..) => true,
            Expression::PickOutput(..) => true,
            Expression::Collect(values, ..) => values
                .iter()
                .fold(true, |valid, value| valid && value.is_valid()),
            Expression::CreationPoint(..) => true,
            Expression::Assert(argument, ..) => argument.is_valid(),
            Expression::Assign { target, value, .. } => {
                (match &**target {
                    Expression::Variable(..) 
                    | Expression::Access{..}
                    | Expression::PickInput(..)
                    | Expression::PickOutput(..) => target.is_valid(),
                    Expression::Proxy {
                        base: proxy_base, ..
                    } => match &**proxy_base {
                        Expression::Variable(..) => target.is_valid(),
                        _ => false,
                    },
                    _ => false,
                }) && value.is_valid()
            }
            Expression::Return(..) => true,
            Expression::FuncCall {
                function,
                setup,
                teardown,
                ..
            } => {
                function.is_valid()
                    && setup
                        .iter()
                        .chain(teardown.iter())
                        .fold(true, |valid, expr| valid && expr.is_valid())
            }
        }
    }
}
