use super::{BaseType, DataType};
use crate::problem::FilePosition;
use crate::vague::structure::{Expression, ScopeId};

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
    Unknown,
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
            KnownData::Unknown
        } else {
            KnownData::Array(
                (0..dims[0])
                    .map(|_| Self::build_array(&dims[1..]))
                    .collect(),
            )
        }
    }

    pub fn new_array(size: usize) -> KnownData {
        KnownData::Array(vec![KnownData::Unknown; size])
    }

    pub fn collect(items: Vec<KnownData>) -> KnownData {
        // TODO: Ensure that each data type is compatible.
        KnownData::Array(items)
    }

    fn get_base_type(&self) -> BaseType {
        match self {
            KnownData::Void => BaseType::Void,
            KnownData::Bool(..) => BaseType::Bool,
            KnownData::Int(..) => BaseType::Int,
            KnownData::Float(..) => BaseType::Float,
            KnownData::DataType(..) => BaseType::DataType_,
            KnownData::Function(..) => BaseType::Function_,
            _ => unreachable!(),
        }
    }

    /// Returns Err if the value is KnownData::Unknown.
    pub fn get_data_type(&self) -> Result<DataType, ()> {
        match self {
            KnownData::Unknown => Err(()),
            KnownData::Array(data) => {
                let first_type = data.iter().find_map(|item| item.get_data_type().ok());
                if let Some(mut data_type) = first_type {
                    data_type.wrap_with_dimension(Expression::Literal(
                        KnownData::Int(data.len() as i64),
                        FilePosition::placeholder(),
                    ));
                    Ok(data_type)
                } else {
                    Err(())
                }
            }
            _ => Ok(DataType::scalar(self.get_base_type())),
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

    fn matches_base_type(&self, base_type: &BaseType) -> bool {
        match self {
            KnownData::Void => {
                if let BaseType::Void = base_type {
                    true
                } else {
                    false
                }
            }
            KnownData::Unknown => false,
            KnownData::Bool(_value) => {
                if let BaseType::Bool = base_type {
                    true
                } else {
                    false
                }
            }
            KnownData::Int(_value) => {
                if let BaseType::Int = base_type {
                    true
                } else {
                    false
                }
            }
            KnownData::Float(_value) => {
                if let BaseType::Float = base_type {
                    true
                } else {
                    false
                }
            }
            KnownData::DataType(_value) => {
                if let BaseType::DataType_ = base_type {
                    true
                } else {
                    false
                }
            }
            KnownData::Function(_value) => {
                if let BaseType::Function_ = base_type {
                    true
                } else {
                    false
                }
            }
            _ => panic!("No other base types exist."),
        }
    }

    pub fn matches_data_type(&self, data_type: &DataType) -> bool {
        match self {
            KnownData::Array(contents) => {
                if data_type.borrow_dimensions().len() == 0 {
                    return false;
                }
                // TODO? check dimensions
                // Check that all the elements have the correct type.
                for item in contents {
                    if !item.matches_data_type(&data_type.clone_and_unwrap(1)) {
                        return false;
                    }
                }
                true
            }
            _ => self.matches_base_type(data_type.borrow_base()),
        }
    }
}

impl Debug for KnownData {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        match self {
            KnownData::Void => write!(formatter, "[void]"),
            KnownData::Unknown => write!(formatter, "[unknown]"),
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
