use super::{Program, VariableId, KnownData, DataType};
use crate::shared::{ProxyMode};

use std::fmt::{self, Debug, Formatter};

#[derive(Clone)]
pub enum ValueBase {
    Literal(KnownData),
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
    pub fn get_type(&self, program: &Program) -> DataType {
        match self {
            Self::Literal(data) => data.get_type(),
            Self::Variable(var) => program[*var].borrow_type().clone(),
        }
    }
}

#[derive(Clone)]
pub struct Value {
    pub base: ValueBase,
    pub dimensions: Vec<(usize, ProxyMode)>,
    pub indexes: Vec<ValueBase>,
}

impl Value {
    pub fn new(base: ValueBase) -> Value {
        Value {
            base,
            dimensions: Vec::new(),
            indexes: Vec::new(),
        }
    }

    pub fn variable(variable: VariableId) -> Value {
        Self::new(ValueBase::Variable(variable))
    }

    pub fn literal(data: KnownData) -> Value {
        Self::new(ValueBase::Literal(data))
    }

    pub fn get_type(&self, program: &Program) -> DataType {
        self.base.get_type(program)
    }
}

impl Debug for Value {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        if self.dimensions.len() == 0 {
            write!(formatter, "{:?}", self.base)?;
        } else {
            write!(formatter, "{{")?;
            for (index, (len, mode)) in self.dimensions.iter().enumerate() {
                match mode {
                    ProxyMode::Keep => write!(formatter, "{}", len)?,
                    ProxyMode::Discard => write!(formatter, "{}>X", len)?,
                    ProxyMode::Collapse => write!(formatter, "{}>1", len)?,
                }
                if index < self.dimensions.len() - 1 {
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
