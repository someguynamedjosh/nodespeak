use std::fmt::{self, Debug, Formatter};

#[derive(Clone, PartialEq)]
pub enum DataType {
    Automatic,
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
            Self::Automatic => write!(formatter, "AUTO"),
            Self::Bool => write!(formatter, "BOOL"),
            Self::Int => write!(formatter, "INT"),
            Self::Float => write!(formatter, "FLOAT"),
            Self::Void => write!(formatter, "VOID"),
            Self::DataType => write!(formatter, "DATA_TYPE"),
            Self::Macro => write!(formatter, "MACRO"),
            Self::Array(size, etype) => write!(formatter, "[{}]{:?}", size, etype),
        }
    }
}
