use super::{Program, VariableId, VariableType};

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
            Self::Bool(..) => VariableType::B8,
        }
    }
}

#[derive(Clone)]
pub enum ValueBase {
    Literal(LiteralData),
    Variable(VariableId),
}

impl Debug for ValueBase {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        match self {
            Self::Literal(data) => write!(formatter, "{:?}", data),
            Self::Variable(var) => write!(formatter, "{:?}", var),
        }
    }
}

impl ValueBase {
    pub fn get_type(&self, program: &Program) -> VariableType {
        match self {
            Self::Literal(data) => data.get_type(),
            Self::Variable(var) => program[*var].get_type(),
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
pub enum ProxyMode {
    /// Use the original array's dimensions.
    Literal,
    /// Discard the index.
    Discard,
    /// No matter what, use index zero.
    Collapse,
}

#[derive(Clone)]
pub struct Value {
    pub base: ValueBase,
    pub proxy: Vec<(ProxyMode, u64)>,
    pub indexes: Vec<Value>,
}

impl Value {
    pub fn new(base: ValueBase) -> Value {
        Value {
            base,
            proxy: Vec::new(),
            indexes: Vec::new(),
        }
    }

    pub fn variable(variable: VariableId) -> Value {
        Self::new(ValueBase::Variable(variable))
    }

    pub fn literal(data: LiteralData) -> Value {
        Self::new(ValueBase::Literal(data))
    }

    pub fn get_type(&self, program: &Program) -> VariableType {
        self.base.get_type(program)
    }
}

impl Debug for Value {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        if self.proxy.len() == 0 {
            write!(formatter, "{:?}", self.base)?;
        } else {
            write!(formatter, "{{")?;
            for (index, (mode, len)) in self.proxy.iter().enumerate() {
                match mode {
                    ProxyMode::Literal => write!(formatter, "{}", len)?,
                    ProxyMode::Collapse => write!(formatter, "{}>1", len)?,
                    ProxyMode::Discard => write!(formatter, "{}>X", len)?,
                }
                if index < self.proxy.len() - 1 {
                    write!(formatter, ", ")?;
                }
            }
            write!(formatter, "}}{:?}", self.base)?;
        }
        for index in self.indexes.iter() {
            write!(formatter, "[{:?}]", index)?;
        }
        write!(formatter, "")
    }
}
