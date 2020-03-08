use super::{GenericProgram, Instruction, NativeType, ProxyMode, VariableId};
use crate::shared::NativeData;

use std::fmt::{self, Debug, Formatter};

#[derive(Clone, PartialEq)]
pub enum Index {
    Constant(usize),
    Variable(VariableId),
}

#[derive(Clone, PartialEq)]
pub enum Value {
    Literal(NativeData),
    VariableAccess {
        variable: VariableId,
        proxy: Vec<ProxyMode>,
        indexes: Vec<Index>,
    },
}

impl Value {
    pub fn get_type<I: Instruction>(&self, program: &GenericProgram<I>) -> NativeType {
        match self {
            Self::Literal(data) => data.get_type(),
            Self::VariableAccess {
                variable, indexes, ..
            } => {
                let base_type = program[*variable].borrow_type();
                base_type.clone_and_unwrap(indexes.len())
            }
        }
    }

    pub fn borrow_proxy(&self) -> Option<&Vec<ProxyMode>> {
        match self {
            Self::Literal(..) => None,
            Self::VariableAccess { proxy, .. } => Some(&proxy),
        }
    }

    pub fn access_element(&self, indices: &[usize]) -> Value {
        match self {
            Self::Literal(data) => {
                if indices.len() == 0 {
                    self.clone()
                } else {
                    if let NativeData::Array(nvec) = data {
                        let slice = nvec.clone_slice(indices);
                        if slice.borrow_dimensions().len() == 0 {
                            Value::Literal(slice.borrow_item(&[]).clone())
                        } else {

                            Value::Literal(NativeData::Array(slice))
                        }
                    } else {
                        unreachable!("Encountered invalid array access.");
                    }
                }
            }
            Self::VariableAccess {
                variable,
                proxy,
                indexes: self_indices,
            } => {
                if proxy.len() == 0 {
                    let mut new_indices = self_indices.clone();
                    for index in indices.iter() {
                        new_indices.push(Index::Constant(*index));
                    }
                    return Self::VariableAccess {
                        variable: *variable,
                        proxy: Vec::new(),
                        indexes: new_indices,
                    };
                }
                assert!(indices.len() <= proxy.len());
                let mut new_indices = self_indices.clone();
                for (index, proxy) in indices.iter().zip(proxy.iter()) {
                    match proxy {
                        ProxyMode::Keep => new_indices.push(Index::Constant(*index)),
                        ProxyMode::Discard => (),
                        ProxyMode::Collapse => new_indices.push(Index::Constant(0)),
                    }
                }
                Self::VariableAccess {
                    variable: *variable,
                    proxy: Vec::from(&proxy[indices.len()..]),
                    indexes: new_indices,
                }
            }
        }
    }
}

impl Debug for Value {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        match self {
            Self::Literal(data) => write!(formatter, "{:?}", data),
            Self::VariableAccess {
                variable,
                proxy,
                indexes,
            } => {
                write!(formatter, "{:?}", variable)?;
                for (index, value) in indexes.iter().enumerate() {
                    match value {
                        Index::Constant(constant) => write!(formatter, "[{}", constant)?,
                        Index::Variable(var) => {
                            write!(formatter, "[{:?}", var)?
                        }
                    }
                    if index < proxy.len() {
                        match proxy[index] {
                            ProxyMode::Keep => (),
                            ProxyMode::Discard => write!(formatter, ">X")?,
                            ProxyMode::Collapse => write!(formatter, ">1")?,
                        }
                    }
                    write!(formatter, "]")?
                }
                write!(formatter, "")
            }
        }
    }
}
