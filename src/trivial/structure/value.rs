use super::{Program, VariableId};
use crate::shared::{NativeData, NativeType, ProxyMode};

use std::fmt::{self, Debug, Formatter};

#[derive(Clone)]
pub enum ValueBase {
    Literal(NativeData),
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
    pub fn get_type(&self, program: &Program) -> NativeType {
        match self {
            Self::Literal(data) => data.get_type(),
            Self::Variable(var) => program[*var].borrow_type().clone(),
        }
    }
}

#[derive(Clone)]
pub struct Value {
    pub base: ValueBase,
    pub proxy: Vec<(ProxyMode, usize)>,
    pub indexes: Vec<ValueBase>,
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

    pub fn literal(data: NativeData) -> Value {
        Self::new(ValueBase::Literal(data))
    }

    pub fn get_type(&self, program: &Program) -> NativeType {
        self.base.get_type(program)
    }

    pub fn borrow_proxy(&self) -> &Vec<(ProxyMode, usize)> {
        &self.proxy
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
                    ProxyMode::Keep => write!(formatter, "{}", len)?,
                    ProxyMode::Discard => write!(formatter, "{}>X", len)?,
                    ProxyMode::Collapse => write!(formatter, "{}>1", len)?,
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
