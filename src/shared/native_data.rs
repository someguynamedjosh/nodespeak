use std::fmt::{self, Debug, Formatter};

use crate::shared::NativeType;
use crate::util::NVec;

#[derive(Clone, PartialEq)]
pub enum NativeData {
    Int(i64),
    Float(f64),
    Bool(bool),
    Array(NVec<NativeData>),
}

impl Debug for NativeData {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        match self {
            Self::Int(value) => write!(formatter, "{}i32", value),
            Self::Float(value) => write!(formatter, "{}f32", value),
            Self::Bool(value) => write!(formatter, "{}b8", if *value { "true" } else { "false" }),
            Self::Array(..) => write!(formatter, "array"),
        }
    }
}

impl NativeData {
    pub fn get_type(&self) -> NativeType {
        match self {
            Self::Int(..) => NativeType::int_scalar(),
            Self::Float(..) => NativeType::float_scalar(),
            Self::Bool(..) => NativeType::bool_scalar(),
            Self::Array(..) => unimplemented!(),
        }
    }

    pub fn binary_data(&self) -> u32 {
        match self {
            Self::Bool(value) => if *value { 1 } else { 0 },
            Self::Int(value) => *value as i32 as u32,
            Self::Float(value) => f32::to_bits(*value as f32),
            Self::Array(..) => unimplemented!()

        }
    }
}
