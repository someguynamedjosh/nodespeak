use std::{cell::RefCell, rc::Rc};

use super::ValuePtr;
use crate::{types::Type, util::Rcrc};

#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    TypeLiteral(Type),
    FloatLiteral(f32),
    IntLiteral(i32),
    BoolLiteral(bool),
    FunctionParameter(ParameterPtr),
    Operation(Operation, Vec<ValuePtr>),
    FunctionCall(ValuePtr, Vec<ValuePtr>),
    Any,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Parameter {
    pub compile_time_only: bool,
    pub name: String,
    pub typee: ValuePtr,
}

#[derive(Clone, Debug)]
pub struct ParameterPtr(Rc<Parameter>);

impl PartialEq for ParameterPtr {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.0, &other.0)
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Operation {
    Add,
    Sub,
    Mul,
    Div,
    Rem,
    Gt,
    Lt,
    Gte,
    Lte,
    Eq,
    Neq,
    And,
    Or,
    Xor,
    Not,
}
