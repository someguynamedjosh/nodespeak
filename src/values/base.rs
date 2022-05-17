use std::{rc::Rc, hash::{Hash, Hasher}, ops::Deref};

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
    MultipleResults(Vec<ValuePtr>),
    // Takes one of several results from an expression
    ExtractResult(ValuePtr, usize),
    Noop,
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

impl Eq for LocalPtr {}

impl Hash for LocalPtr {
    fn hash<H: Hasher>(&self, state: &mut H) {
        Rc::as_ptr(&self.0).hash(state);
    }
}

impl Deref for LocalPtr {
    type Target = Local;

    fn deref(&self) -> &Self::Target {
        &*self.0
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
    Typeof
}
