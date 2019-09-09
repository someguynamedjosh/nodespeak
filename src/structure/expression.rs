use crate::structure::{KnownData, VariableId};
use std::borrow::Borrow;
use std::fmt::{self, Debug, Formatter};

#[derive(Clone, PartialEq)]
pub enum Expression {
    Literal(KnownData),
    Variable(VariableId),
    Access {
        base: Box<Expression>,
        indexes: Vec<Expression>,
    },
    InlineReturn,

    Add(Box<Expression>, Box<Expression>),
    Subtract(Box<Expression>, Box<Expression>),
    Multiply(Box<Expression>, Box<Expression>),
    Divide(Box<Expression>, Box<Expression>),
    IntDiv(Box<Expression>, Box<Expression>),
    Reciprocal(Box<Expression>),
    Modulo(Box<Expression>, Box<Expression>),
    Power(Box<Expression>, Box<Expression>),

    And(Box<Expression>, Box<Expression>),
    Or(Box<Expression>, Box<Expression>),
    Xor(Box<Expression>, Box<Expression>),
    Not(Box<Expression>),
    BAnd(Box<Expression>, Box<Expression>),
    BOr(Box<Expression>, Box<Expression>),
    BXor(Box<Expression>, Box<Expression>),
    BNot(Box<Expression>),

    Equal(Box<Expression>, Box<Expression>),
    NotEqual(Box<Expression>, Box<Expression>),
    LessThan(Box<Expression>, Box<Expression>),
    GreaterThan(Box<Expression>, Box<Expression>),
    LessThanOrEqual(Box<Expression>, Box<Expression>),
    GreaterThanOrEqual(Box<Expression>, Box<Expression>),

    Collect(Vec<Expression>),

    Assert(Box<Expression>),
    Assign {
        target: Box<Expression>,
        value: Box<Expression>,
    },
    Return,

    FuncCall {
        function: Box<Expression>,
        inputs: Vec<Expression>,
        outputs: Vec<Expression>,
    },
}

impl Debug for Expression {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        match self {
            Expression::Literal(value) => write!(formatter, "{:?}", value),
            Expression::Variable(id) => write!(formatter, "{:?}", id),
            Expression::Access { base, indexes } => {
                write!(formatter, "{:?}", base)?;
                for index in indexes.iter() {
                    write!(formatter, "[{:?}]", index)?;
                }
                write!(formatter, "")
            }
            Expression::InlineReturn => write!(formatter, "inline"),

            Expression::Add(v1, v2) => write!(formatter, "({:?} + {:?})", v1, v2),
            Expression::Subtract(v1, v2) => write!(formatter, "({:?} - {:?})", v1, v2),
            Expression::Multiply(v1, v2) => write!(formatter, "({:?} * {:?})", v1, v2),
            Expression::Divide(v1, v2) => write!(formatter, "({:?} / {:?})", v1, v2),
            Expression::IntDiv(v1, v2) => write!(formatter, "({:?} // {:?})", v1, v2),
            Expression::Reciprocal(v1) => write!(formatter, "(reciprocal of {:?})", v1),
            Expression::Modulo(v1, v2) => write!(formatter, "({:?} % {:?})", v1, v2),
            Expression::Power(v1, v2) => write!(formatter, "({:?} ** {:?})", v1, v2),

            Expression::And(v1, v2) => write!(formatter, "({:?} and {:?})", v1, v2),
            Expression::Or(v1, v2) => write!(formatter, "({:?} or {:?})", v1, v2),
            Expression::Xor(v1, v2) => write!(formatter, "({:?} xor {:?})", v1, v2),
            Expression::Not(v1) => write!(formatter, "(not {:?})", v1),
            Expression::BAnd(v1, v2) => write!(formatter, "({:?} band {:?})", v1, v2),
            Expression::BOr(v1, v2) => write!(formatter, "({:?} bor {:?})", v1, v2),
            Expression::BXor(v1, v2) => write!(formatter, "({:?} bxor {:?})", v1, v2),
            Expression::BNot(v1) => write!(formatter, "(bnot {:?})", v1),

            Expression::Equal(v1, v2) => write!(formatter, "({:?} == {:?})", v1, v2),
            Expression::NotEqual(v1, v2) => write!(formatter, "({:?} != {:?})", v1, v2),
            Expression::LessThan(v1, v2) => write!(formatter, "({:?} < {:?})", v1, v2),
            Expression::GreaterThan(v1, v2) => write!(formatter, "({:?} > {:?})", v1, v2),
            Expression::LessThanOrEqual(v1, v2) => write!(formatter, "({:?} <= {:?})", v1, v2),
            Expression::GreaterThanOrEqual(v1, v2) => write!(formatter, "({:?} >= {:?})", v1, v2),

            Expression::Collect(values) => {
                write!(formatter, "[")?;
                for value in values.iter() {
                    write!(formatter, "{:?}, ", value)?;
                }
                write!(formatter, "]")
            }

            Expression::Assert(value) => write!(formatter, "assert {:?};", value),
            Expression::Assign { target, value } => {
                write!(formatter, "{:?} = {:?};", target, value)
            }
            Expression::Return => write!(formatter, "return;"),

            Expression::FuncCall {
                function,
                inputs,
                outputs,
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
