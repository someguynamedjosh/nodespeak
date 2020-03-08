use super::{DataType, Program};
use crate::problem::FilePosition;
use crate::resolved::structure::{KnownData, ScopeId, VariableId};
use crate::shared::ProxyMode;
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
pub enum Expression {
    Variable(VariableId, FilePosition),
    Literal(KnownData, FilePosition),
    InlineReturn(FilePosition),
    Proxy {
        base: Box<Expression>,
        dimensions: Vec<(usize, ProxyMode)>,
    },
    Access {
        base: Box<Expression>,
        indexes: Vec<Expression>,
        position: FilePosition,
    },

    UnaryOperation(UnaryOperator, Box<Expression>, FilePosition),
    BinaryOperation(
        Box<Expression>,
        BinaryOperator,
        Box<Expression>,
        FilePosition,
    ),
    Collect(Vec<Expression>, FilePosition),

    Assert(Box<Expression>, FilePosition),
    Assign {
        target: Box<Expression>,
        value: Box<Expression>,
        position: FilePosition,
    },
    Return(FilePosition),

    FuncCall {
        function: ScopeId,
        position: FilePosition,
    },
}

impl Expression {
    // TODO: Validation function.
    pub fn get_type(&self, program: &Program) -> DataType {
        match self {
            Expression::Variable(id, ..) => program[*id].borrow_data_type().clone(),
            Expression::Literal(data, ..) => data.get_data_type(),
            Expression::InlineReturn(..) => unimplemented!(),
            Expression::Proxy { base, dimensions } => {
                let base_type = base.get_type(program);
                let base_dimensions = base_type.borrow_dimensions();
                let mut new_dimensions = Vec::new();
                let mut current_base_dimension = 0;
                for (new_size, proxy_mode) in dimensions {
                    match proxy_mode {
                        ProxyMode::Keep => {
                            debug_assert!(new_size == &base_dimensions[current_base_dimension].0);
                            new_dimensions.push(base_dimensions[current_base_dimension].clone());
                            current_base_dimension += 1;
                        }
                        ProxyMode::Collapse | ProxyMode::Discard => {
                            new_dimensions.push((*new_size, *proxy_mode));
                        }
                    }
                }
                DataType::proxy(base_type.borrow_base().clone(), new_dimensions)
            }
            Expression::Access { .. } => unimplemented!(),
            Expression::UnaryOperation(_, value, ..) => value.get_type(program),
            Expression::BinaryOperation(a, _, b, ..) => {
                debug_assert!(a.get_type(program).equivalent(&b.get_type(program)));
                a.get_type(program)
            }
            Expression::Collect(values, ..) => {
                debug_assert!(values.len() > 0);
                let mut element_type = values[0].get_type(program);
                for element in values {
                    debug_assert!(element.get_type(program).equivalent(&element_type));
                }
                element_type.wrap_with_dimension(values.len() as usize);
                element_type
            }
            Expression::Assert(..) => unimplemented!(),
            Expression::Assign { .. } => unimplemented!(),
            Expression::Return { .. } => unimplemented!(),
            Expression::FuncCall { .. } => unimplemented!(),
        }
    }
}

impl Debug for Expression {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        match self {
            Expression::Variable(var_id, ..) => write!(formatter, "{:?}", var_id),
            Expression::Literal(value, ..) => write!(formatter, "{:?}", value),
            Expression::InlineReturn(..) => write!(formatter, "inline return"),
            Expression::Proxy {
                base, dimensions, ..
            } => {
                write!(formatter, "{{")?;
                for (index, dimension) in dimensions.iter().enumerate() {
                    if index < dimensions.len() - 1 {
                        write!(formatter, ", ")?;
                    }
                }
                write!(formatter, "}}{:?}", base)
            }
            Expression::Access { base, indexes, .. } => {
                write!(formatter, "{:?}", base)?;
                for index in indexes {
                    write!(formatter, "[{:?}]", index)?;
                }
                write!(formatter, "")
            }

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

            Expression::Assert(value, ..) => write!(formatter, "assert {:?};", value),
            Expression::Assign { target, value, .. } => {
                write!(formatter, "{:?} = {:?};", target, value)
            }
            Expression::Return(..) => write!(formatter, "return;"),

            Expression::FuncCall { function, .. } => write!(formatter, "call {:?} ", function),
        }
    }
}

impl Expression {
    pub fn clone_position(&self) -> FilePosition {
        match self {
            Expression::Variable(_, position)
            | Expression::Literal(_, position)
            | Expression::InlineReturn(position)
            | Expression::Access { position, .. }
            | Expression::UnaryOperation(_, _, position)
            | Expression::BinaryOperation(_, _, _, position)
            | Expression::Collect(_, position)
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
        // TODO: Access and Proxy expressions.
        match self {
            Expression::Variable(..) => true,
            Expression::Literal(..) => true,
            Expression::InlineReturn(..) => true,
            Expression::Proxy { .. } => true, // TODO: Actual implementation.
            Expression::Access { .. } => true, // TODO: Actual implementation.
            Expression::UnaryOperation(_, operand, ..) => operand.is_valid(),
            Expression::BinaryOperation(op1, _, op2, ..) => op1.is_valid() && op2.is_valid(),
            Expression::Collect(values, ..) => values
                .iter()
                .fold(true, |valid, value| valid && value.is_valid()),
            Expression::Assert(argument, ..) => argument.is_valid(),
            Expression::Assign { target, value, .. } => {
                (match &**target {
                    Expression::Variable(..) => true,
                    Expression::Access { .. } => target.is_valid(),
                    _ => false,
                }) && value.is_valid()
            }
            Expression::Return(..) => true,
            Expression::FuncCall { .. } => true,
        }
    }
}
