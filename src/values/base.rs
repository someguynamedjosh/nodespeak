use std::rc::Rc;

use super::ValuePtr;
use crate::types::Type;

#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    TypeLiteral(Type),
    FloatLiteral(f32),
    IntLiteral(i32),
    BoolLiteral(bool),
    Local(LocalPtr),
    Operation(Operation, Vec<ValuePtr>),
    Assignment {
        base: ValuePtr,
        targets: Vec<LocalPtr>,
    },
    Function {
        inputs: Vec<LocalPtr>,
        outputs: Vec<LocalPtr>,
        locals: Vec<LocalPtr>,
        body: Vec<ValuePtr>,
    },
    FunctionCall(ValuePtr, Vec<ValuePtr>),
    Any,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Local {
    pub compile_time_only: bool,
    pub name: String,
    pub typee: ValuePtr,
}

#[derive(Clone, Debug)]
pub struct LocalPtr(Rc<Local>);

impl PartialEq for LocalPtr {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.0, &other.0)
    }
}

impl LocalPtr {
    pub fn new(local: Local) -> Self {
        Self(Rc::new(local))
    }

    pub fn ptr_clone(&self) -> Self {
        Self(Rc::clone(&self.0))
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
