use crate::vague::structure::VariableId;

use std::fmt::{self, Debug, Formatter};

#[derive(Clone, PartialEq)]
pub enum DataType {
    Automatic,
    Dynamic(VariableId),
    LoadTemplateParameter(VariableId),
    Bool,
    Int,
    Float,
    Void,
    DataType,
    Macro,
    Array(usize, Box<DataType>),
}

impl DataType {
    pub fn is_automatic(&self) -> bool {
        match self {
            Self::Automatic => true,
            Self::Array(_, element_type) => element_type.is_automatic(),
            _ => false,
        }
    }

    pub fn equivalent(&self, other: &Self) -> bool {
        match self {
            // If it's a basic type, just check if it is equal to the other one.
            Self::Automatic
            | Self::Bool
            | Self::Int
            | Self::Float
            | Self::Void
            | Self::DataType
            | Self::Macro => self == other,
            // TODO?: better equivalency for this.
            Self::Dynamic(..) | Self::LoadTemplateParameter(..) => match other {
                Self::Dynamic(..) | Self::LoadTemplateParameter(..) => true,
                _ => false,
            },
            Self::Array(my_size, my_etype) => {
                if let Self::Array(size, etype) = other {
                    my_size == size && my_etype.equivalent(etype)
                } else {
                    false
                }
            }
        }
    }
}

impl Debug for DataType {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        match self {
            Self::Automatic => write!(formatter, "Auto"),
            Self::Dynamic(_var) => write!(formatter, "Unresolved"),
            Self::LoadTemplateParameter(_var) => write!(formatter, "Unresolved"),
            Self::Bool => write!(formatter, "Bool"),
            Self::Int => write!(formatter, "Int"),
            Self::Float => write!(formatter, "Float"),
            Self::Void => write!(formatter, "Void"),
            Self::DataType => write!(formatter, "DataType"),
            Self::Macro => write!(formatter, "Macro"),
            Self::Array(size, etype) => write!(formatter, "[{}]{:?}", size, etype),
        }
    }
}
