use super::{BaseType, DataType, KnownData};

use std::fmt::{self, Debug, Formatter};

#[derive(Clone)]
pub struct Variable {
    data_type: DataType,
}

impl Variable {
    pub fn new(data_type: DataType) -> Variable {
        Variable {
            data_type
        }
    }

    pub fn bool() -> Variable {
        Self::new(DataType::scalar(BaseType::Bool))
    }

    pub fn int() -> Variable {
        Self::new(DataType::scalar(BaseType::Int))
    }

    pub fn float() -> Variable {
        Self::new(DataType::scalar(BaseType::Float))
    }

    pub fn borrow_data_type(&self) -> &DataType {
        &self.data_type
    }
}