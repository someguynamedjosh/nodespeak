use super::DataType;
use crate::problem::FilePosition;
use crate::vague::structure::ScopeId;

use std::fmt::{self, Debug, Formatter};

#[derive(Clone, Debug)]
pub struct FunctionData {
    body: ScopeId,
    header: FilePosition,
}

impl PartialEq for FunctionData {
    fn eq(&self, other: &Self) -> bool {
        self.body == other.body
    }
}

impl FunctionData {
    pub fn new(body: ScopeId, header: FilePosition) -> FunctionData {
        FunctionData { body, header }
    }

    pub fn set_header(&mut self, new_header: FilePosition) {
        self.header = new_header;
    }

    pub fn get_header(&self) -> &FilePosition {
        &self.header
    }

    pub fn get_body(&self) -> ScopeId {
        self.body
    }
}

#[derive(Clone, PartialEq)]
pub enum KnownData {
    Void,
    Bool(bool),
    Int(i64),
    Float(f64),
    DataType(DataType),
    Function(FunctionData),
    Array(Vec<KnownData>),
}

impl KnownData {
    pub fn build_array(dims: &[usize]) -> KnownData {
        if dims.len() == 0 {
            KnownData::Void
        } else {
            KnownData::Array(
                (0..dims[0])
                    .map(|_| Self::build_array(&dims[1..]))
                    .collect(),
            )
        }
    }

    pub fn new_array(size: usize) -> KnownData {
        KnownData::Array(vec![KnownData::Void; size])
    }

    pub fn collect(items: Vec<KnownData>) -> KnownData {
        // TODO: Ensure that each data type is compatible.
        KnownData::Array(items)
    }

    pub fn get_data_type(&self) -> DataType {
        match self {
            KnownData::Array(data) => {
                assert!(data.len() > 0);
                let first_type = data[0].get_data_type();
                DataType::Array(data.len(), Box::new(first_type))
            }
            KnownData::Void => DataType::Void,
            KnownData::Bool(..) => DataType::Bool,
            KnownData::Int(..) => DataType::Int,
            KnownData::Float(..) => DataType::Float,
            KnownData::DataType(..) => DataType::DataType_,
            KnownData::Function(..) => DataType::Function_,
        }
    }

    pub fn require_bool(&self) -> bool {
        match self {
            KnownData::Bool(value) => *value,
            _ => panic!("Expected data to be a bool."),
        }
    }

    pub fn require_int(&self) -> i64 {
        match self {
            KnownData::Int(value) => *value,
            _ => panic!("Expected data to be an int."),
        }
    }

    pub fn require_float(&self) -> f64 {
        match self {
            KnownData::Float(value) => *value,
            _ => panic!("Expected data to be a float."),
        }
    }

    pub fn require_data_type(&self) -> &DataType {
        match self {
            KnownData::DataType(value) => value,
            _ => panic!("Expected data to be a data type."),
        }
    }

    pub fn require_function(&self) -> &FunctionData {
        match self {
            KnownData::Function(value) => value,
            _ => panic!("Expected data to be a function."),
        }
    }

    pub fn require_array(&self) -> &Vec<KnownData> {
        match self {
            KnownData::Array(value) => value,
            _ => panic!("Expected data to be an array."),
        }
    }

    pub fn require_array_mut(&mut self) -> &mut Vec<KnownData> {
        match self {
            KnownData::Array(value) => value,
            _ => panic!("Expected data to be an array."),
        }
    }

    pub fn matches_data_type(&self, data_type: &DataType) -> bool {
        match self {
            KnownData::Array(contents) => {
                if let DataType::Array(len, etype) = data_type {
                    if len == &contents.len() {
                        assert!(contents.len() > 0);
                        return contents[0].matches_data_type(etype);
                    }
                }
                false
            }
            KnownData::Bool(..) => data_type == &DataType::Bool,
            KnownData::Int(..) => data_type == &DataType::Int,
            KnownData::Float(..) => data_type == &DataType::Float,
            KnownData::Function(..) => data_type == &DataType::Function_,
            KnownData::DataType(..) => data_type == &DataType::DataType_,
            KnownData::Void => data_type == &DataType::Void,
        }
    }
}

impl Debug for KnownData {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        match self {
            KnownData::Void => write!(formatter, "[void]"),
            KnownData::Bool(value) => {
                write!(formatter, "{}", if *value { "true" } else { "false" })
            }
            KnownData::Int(value) => write!(formatter, "{}", value),
            KnownData::Float(value) => write!(formatter, "{}", value),
            KnownData::Array(values) => {
                write!(formatter, "[")?;
                if values.len() > 0 {
                    for value in &values[..values.len() - 1] {
                        write!(formatter, "{:?}, ", value)?;
                    }
                    write!(formatter, "{:?}", values[values.len() - 1])?;
                }
                write!(formatter, "]")
            }
            KnownData::DataType(value) => write!(formatter, "{:?}", value),
            // TODO: Implement function formatter.
            KnownData::Function(value) => {
                write!(formatter, "function with body at {:?}", value.body)
            }
        }
    }
}
