use crate::resolved::structure::{DataType, KnownData, Program, VariableId};

use std::fmt::{self, Debug, Formatter};

#[derive(Clone, PartialEq)]
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
    pub fn get_data_type(&self, program: &Program) -> DataType {
        match self {
            Self::Literal(data) => data.get_data_type(),
            Self::Variable(var) => program[*var].borrow_data_type().clone(),
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

#[derive(Clone, PartialEq)]
pub struct Value {
    pub base: ValueBase,
    pub proxy: Vec<(i64, ProxyMode)>,
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

    pub fn literal(data: KnownData) -> Value {
        Self::new(ValueBase::Literal(data))
    }

    pub fn is_variable(&self) -> bool {
        match self.base {
            ValueBase::Literal(..) => false,
            ValueBase::Variable(..) => true,
        }
    }

    pub fn get_data_type(&self, program: &Program) -> DataType {
        // TODO: Account for proxies and indexes.
        self.base.get_data_type(program)
    }
}

impl Debug for Value {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        if self.proxy.len() == 0 {
            write!(formatter, "{:?}", self.base)?;
        } else {
            write!(formatter, "(")?;
            for (len, mode) in self.proxy.iter() {
                match mode {
                    ProxyMode::Literal => write!(formatter, "{{{}}}", len)?,
                    ProxyMode::Collapse => write!(formatter, "{{{}>1}}", len)?,
                    ProxyMode::Discard => write!(formatter, "{{{}x}}", len)?,
                }
            }
            write!(formatter, "{:?})", self.base)?;
        }
        for index in self.indexes.iter() {
            write!(formatter, "[{:?}]", index)?;
        }
        write!(formatter, "")
    }
}
