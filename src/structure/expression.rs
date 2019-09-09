use crate::structure::{KnownData, VariableId};

#[derive(Clone, Debug, PartialEq)]
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
    Not(Box<Expression>, Box<Expression>),
    BAnd(Box<Expression>, Box<Expression>),
    BOr(Box<Expression>, Box<Expression>),
    BXor(Box<Expression>, Box<Expression>),
    BNot(Box<Expression>, Box<Expression>),

    Equal(Box<Expression>, Box<Expression>),
    NotEqual(Box<Expression>, Box<Expression>),
    LessThan(Box<Expression>, Box<Expression>),
    GreaterThan(Box<Expression>, Box<Expression>),
    LessThanOrEqual(Box<Expression>, Box<Expression>),
    GreaterThanOrEqual(Box<Expression>, Box<Expression>),

    Collect(Vec<Expression>),

    Assign {
        target: Box<Expression>,
        value: Box<Expression>,
    },

    FuncCall {
        function: Box<Expression>,
        inputs: Vec<Expression>,
        outputs: Vec<Expression>,
    },
    Return,
}
