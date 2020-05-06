use std::fmt::{self, Debug, Formatter};

use super::ScopeResolver;
use crate::resolved::structure as o;
use crate::vague::structure as i;

#[derive(Clone, PartialEq)]
pub enum DataType {
    Bool,
    Int,
    Float,
    Macro,
    DataType,
    Void,
    Automatic,
    Array(usize, Box<DataType>),
}

impl DataType {
    pub fn make_array(dims: &[usize], base: Self) -> Self {
        if dims.len() > 0 {
            Self::Array(dims[0], Box::new(Self::make_array(&dims[1..], base)))
        } else {
            base
        }
    }

    pub fn resolve(&self) -> Option<o::DataType> {
        match self {
            Self::Array(len, base) => base
                .resolve()
                .map(|base| o::DataType::Array(*len, Box::new(base))),
            Self::Bool => Some(o::DataType::Bool),
            Self::Int => Some(o::DataType::Int),
            Self::Float => Some(o::DataType::Float),
            _ => None,
        }
    }

    fn collect_dims_impl(&self, dims: &mut Vec<usize>) {
        if let Self::Array(size, btype) = self {
            dims.push(*size);
            btype.collect_dims_impl(dims);
        }
    }

    pub fn collect_dims(&self) -> Vec<usize> {
        let mut dims = Vec::new();
        self.collect_dims_impl(&mut dims);
        dims
    }

    pub fn is_automatic(&self) -> bool {
        match self {
            Self::Automatic => true,
            Self::Array(_, etype) => etype.is_automatic(),
            _ => false,
        }
    }

    pub fn with_different_base(&self, new_base: DataType) -> Self {
        match self {
            Self::Array(size, etyp) => {
                Self::Array(*size, Box::new(etyp.with_different_base(new_base)))
            }
            _ => new_base,
        }
    }
}

impl Debug for DataType {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        match self {
            DataType::Bool => write!(formatter, "BOOL"),
            DataType::Int => write!(formatter, "INT"),
            DataType::Float => write!(formatter, "FLOAT"),
            DataType::Macro => write!(formatter, "MACRO"),
            DataType::DataType => write!(formatter, "DATA_TYPE"),
            DataType::Void => write!(formatter, "VOID"),
            DataType::Automatic => write!(formatter, "AUTO"),
            DataType::Array(len, etype) => write!(formatter, "[{}]{:?}", len, etype),
        }
    }
}

impl<'a> ScopeResolver<'a> {
    pub fn resolve_data_type_partially(&mut self, source: i::DataType) -> DataType {
        match source {
            i::DataType::Automatic => DataType::Automatic,
            i::DataType::Bool => DataType::Bool,
            i::DataType::Int => DataType::Int,
            i::DataType::Float => DataType::Float,
            i::DataType::DataType => DataType::DataType,
            i::DataType::Macro => DataType::Macro,
            i::DataType::Void => DataType::Void,
            i::DataType::Array(len, etype) => DataType::Array(
                len,
                Box::new(self.resolve_data_type_partially(etype.as_ref().clone())),
            ),
        }
    }
}
