use super::VariableId;

use std::fmt::{self, Debug, Formatter};

#[derive(Clone, PartialEq)]
pub enum LiteralData {
    Int(i64),
    Float(f64),
    Bool(bool),
}

impl LiteralData {
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
pub enum Value {
    Literal(LiteralData),
    VariableAccess {
        variable: VariableId,
        // Variable for offset and u64 to multiply it by.
        indexes: Vec<(VariableId, u64)>,
        offset: u64,
        length: u64,
    },
}

impl Debug for Value {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        match self {
            Self::Literal(data) => write!(formatter, "{:?}", data),
            Self::VariableAccess {
                variable,
                indexes,
                offset,
                length,
            } => {
                write!(formatter, "{:?}[", variable)?;
                for index in indexes {
                    write!(formatter, "{:?}*{:?} + ", index.1, index.0)?;
                }
                if *offset > 0 || indexes.len() > 0 {
                    write!(formatter, "{:?} ", offset)?;
                }
                write!(formatter, "len={:?}]", length)
            }
        }
    }
}
