use std::{
    hash::{Hash, Hasher},
    ops::Deref,
    rc::Rc,
};

use super::ValuePtr;

#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    BuiltinType(BuiltinType),
    BuiltinOp(BuiltinOp),
    Noop,
    Any,
    Malformed,
    FloatLiteral(f32),
    IntLiteral(i32),
    BoolLiteral(bool),
    ArrayLiteral {
        elements: Vec<ValuePtr>,
        dims: Vec<ValuePtr>,
    },
    Local(LocalPtr),
    Assignment {
        base: ValuePtr,
        target: LocalPtr,
    },
    Function {
        inputs: Vec<LocalPtr>,
        outputs: Vec<LocalPtr>,
        locals: Vec<LocalPtr>,
        body: Vec<ValuePtr>,
    },
    FunctionCall(ValuePtr, Vec<ValuePtr>, usize),
}

#[derive(Clone, Debug, PartialEq)]
pub enum BuiltinType {
    Int,
    Float,
    Bool,
    Type,
    Array {
        eltype: ValuePtr,
        dims: Vec<ValuePtr>,
    },
    InSet {
        eltype: ValuePtr,
        elements: Vec<ValuePtr>,
    },
    Function {
        inputs: Vec<ValuePtr>,
        outputs: Vec<ValuePtr>,
    },
    Malformed,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum BuiltinOp {
    Add,
    Sub,
    Mul,
    Div,
    Rem,

    Min,
    Max,

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

    Typeof,
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
    Typeof,
}
