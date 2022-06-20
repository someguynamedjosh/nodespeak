use std::{
    hash::{Hash, Hasher},
    ops::Deref,
    rc::Rc,
};

use super::ValuePtr;

#[derive(Clone, Debug, PartialEq)]
pub struct Index {
    pub indices: Vec<ValuePtr>,
    pub eight_wide_mode: bool,
}

impl Index {
    pub fn deep_clone(&self) -> Self {
        Self {
            indices: self.indices.iter().map(ValuePtr::deep_clone).collect(),
            eight_wide_mode: self.eight_wide_mode,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Statement {
    Assignment {
        base: ValuePtr,
        index: Option<Index>,
        target: LocalPtr,
    },
    Declaration(LocalPtr),
    Noop,
}

impl Statement {
    pub fn deep_clone(&self) -> Self {
        match self {
            Statement::Assignment {
                base,
                index,
                target,
            } => Statement::Assignment {
                base: base.deep_clone(),
                index: index.clone(),
                target: target.ptr_clone(),
            },
            Statement::Declaration(local) => Statement::Declaration(local.ptr_clone()),
            Statement::Noop => Statement::Noop,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    BuiltinType(BuiltinType),
    BuiltinOp(BuiltinOp),
    Malformed,
    FloatLiteral(f32),
    IntLiteral(i32),
    BoolLiteral(bool),
    ArrayLiteral {
        elements: Vec<ValuePtr>,
        dims: Vec<ValuePtr>,
    },
    Local(LocalPtr),
    Function {
        inputs: Vec<LocalPtr>,
        outputs: Vec<LocalPtr>,
        locals: Vec<LocalPtr>,
        body: Vec<Statement>,
    },
    FunctionCall(ValuePtr, Vec<ValuePtr>, usize),
}

impl Value {
    pub fn deep_clone(&self) -> Self {
        match &self {
            Value::BuiltinType(_)
            | Value::BuiltinOp(_)
            | Value::Malformed
            | Value::FloatLiteral(_)
            | Value::IntLiteral(_)
            | Value::BoolLiteral(_)
            | Value::Local(_) => self.clone(),
            Value::ArrayLiteral { elements, dims } => Self::ArrayLiteral {
                elements: elements.iter().map(ValuePtr::deep_clone).collect(),
                dims: dims.iter().map(ValuePtr::deep_clone).collect(),
            },
            Value::Function {
                inputs,
                outputs,
                locals,
                body,
            } => Value::Function {
                inputs: inputs.clone(),
                outputs: outputs.clone(),
                locals: locals.clone(),
                body: body.iter().map(Statement::deep_clone).collect(),
            },
            Value::FunctionCall(base, args, output) => Value::FunctionCall(
                base.deep_clone(),
                args.iter().map(ValuePtr::deep_clone).collect(),
                *output,
            ),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum BuiltinType {
    Any,
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

    Cast,
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
