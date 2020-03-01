use super::{GenericProgram, Instruction, NativeType, ProxyMode, VariableId};

use std::fmt::{self, Debug, Formatter};

#[derive(Clone, PartialEq)]
pub enum LiteralData {
    Int(i64),
    Float(f64),
    Bool(bool),
}

impl LiteralData {
    pub fn get_type(&self) -> NativeType {
        match self {
            Self::Int(..) => NativeType::int_scalar(),
            Self::Float(..) => NativeType::float_scalar(),
            Self::Bool(..) => NativeType::bool_scalar(),
        }
    }

    pub fn binary_data(&self) -> u32 {
        match self {
            Self::Bool(data) => {
                if *data {
                    1
                } else {
                    0
                }
            }
            // TODO: Check for signed values.
            Self::Int(data) => *data as i32 as u32,
            Self::Float(data) => f32::to_bits(*data as f32),
        }
    }
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

#[derive(Clone, PartialEq)]
pub enum Index {
    Constant(u64),
    Variable { base: VariableId, multiplier: u64 },
}

#[derive(Clone, PartialEq)]
pub enum Value {
    Literal(LiteralData),
    VariableAccess {
        variable: VariableId,
        proxy: Vec<ProxyMode>,
        // Variable for offset and u64 to multiply it by.
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

    pub fn access_element(&self, indices: &[u64]) -> Value {
        match self {
            Self::Literal(..) => unimplemented!(),
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
                        Index::Variable { base, multiplier } => {
                            write!(formatter, "[{:?}*{:?}", base, multiplier)?
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
