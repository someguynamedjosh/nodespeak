use super::{Program, VariableType, VariableId};

use std::fmt::{self, Debug, Formatter};

#[derive(Clone)]
pub enum LiteralData {
    Int(i64),
    Float(f64),
    Bool(bool),
}

impl Debug for LiteralData {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        match self {
            Self::Int(value) => write!(formatter, "{}i32", value),
            Self::Float(value) => write!(formatter, "{}f32", value),
            Self::Bool(value) => write!(formatter, "{}b8", if *value { "true" } else { "false" }),
        }
    }
}

impl LiteralData {
    pub fn get_type(&self) -> VariableType {
        match self {
            Self::Int(..) => VariableType::I32,
            Self::Float(..) => VariableType::F32,
            Self::Bool(..) => VariableType::B8
        }
    }
}

#[derive(Clone)]
pub enum Value {
    Literal(LiteralData),
    Variable(VariableId),
    Index(VariableId, Vec<Value>),
}

impl Debug for Value {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        match self {
            Self::Literal(data) => write!(formatter, "{:?}", data),
            Self::Variable(var) => write!(formatter, "{:?}", var),
            Self::Index(..) => unimplemented!()
        }
    }
}

impl Value {
    pub fn get_type(&self, program: &Program) -> VariableType {
        match self {
            Self::Literal(data) => data.get_type(),
            Self::Variable(var)
            | Self::Index(var, ..) => program[*var].get_type()
        }
    }
}