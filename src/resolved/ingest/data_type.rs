use std::fmt::{self, Debug, Formatter};

use super::ScopeSimplifier;
use crate::resolved::structure as o;
use crate::vague::structure as i;

#[derive(Clone, PartialEq)]
pub enum DataType {
    Bool,
    Int,
    Float,
    Function_,
    DataType_,
    Void,
    Automatic,
    Array(usize, Box<DataType>),
}

impl DataType {
    pub fn resolve(&self) -> Option<o::DataType> {
        match self {
            Self::Array(len, base) => base
                .resolve()
                .map(|base| o::DataType::Array(*len, Box::new(base))),
            Self::Bool => Some(o::DataType::Bool),
            Self::Int => Some(o::DataType::Int),
            Self::Float => Some(o::DataType::Float),
            _ => None
        }
    }
}

impl Debug for DataType {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        match self {
            DataType::Bool => write!(formatter, "Bool"),
            DataType::Int => write!(formatter, "Int"),
            DataType::Float => write!(formatter, "Float"),
            DataType::Function_ => write!(formatter, "Function_"),
            DataType::DataType_ => write!(formatter, "DataType_"),
            DataType::Void => write!(formatter, "Void"),
            DataType::Automatic => write!(formatter, "Automatic"),
            DataType::Array(len, etype) => write!(formatter, "[{}]{:?}", len, etype),
        }
    }
}

impl<'a> ScopeSimplifier<'a> {
    pub fn input_to_intermediate_type(&mut self, source: i::DataType) -> DataType {
        match source {
            i::DataType::Automatic => DataType::Automatic,
            i::DataType::Dynamic(..) => unimplemented!("TODO: Dynamic types."),
            i::DataType::LoadTemplateParameter(..) => unimplemented!("TODO: Template parameters."),
            i::DataType::Bool => DataType::Bool,
            i::DataType::Int => DataType::Int,
            i::DataType::Float => DataType::Float,
            i::DataType::DataType_ => DataType::DataType_,
            i::DataType::Function_ => DataType::Function_,
            i::DataType::Void => DataType::Void,
            i::DataType::Array(len, etype) => DataType::Array(
                len,
                Box::new(self.input_to_intermediate_type(etype.as_ref().clone())),
            ),
        }
    }
}
