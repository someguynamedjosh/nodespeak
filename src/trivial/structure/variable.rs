use std::fmt::{self, Debug, Formatter};

#[derive(Clone, Copy, PartialEq)]
pub enum VariableType {
    F32,
    I32,
    B8,
}

impl Debug for VariableType {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        write!(
            formatter,
            "{}",
            match self {
                Self::F32 => "f32",
                Self::I32 => "i32",
                Self::B8 => "b8",
            }
        )
    }
}

#[derive(Clone)]
pub struct Variable {
    variable_type: VariableType,
    dimensions: Vec<u64>,
}

impl Debug for Variable {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        for dim in self.dimensions.iter() {
            write!(formatter, "[{}]", dim)?;
        }
        write!(formatter, "{:?}", self.variable_type)
    }
}

impl Variable {
    pub fn new(typ: VariableType) -> Variable {
        Variable {
            variable_type: typ,
            dimensions: vec![],
        }
    }

    pub fn f32() -> Variable {
        Self::new(VariableType::F32)
    }

    pub fn i32() -> Variable {
        Self::new(VariableType::I32)
    }

    pub fn new_array(typ: VariableType, dimensions: Vec<u64>) -> Variable {
        Variable {
            variable_type: typ,
            dimensions,
        }
    }

    pub fn f32_array(dimensions: Vec<u64>) -> Variable {
        Self::new_array(VariableType::F32, dimensions)
    }

    pub fn i32_array(dimensions: Vec<u64>) -> Variable {
        Self::new_array(VariableType::I32, dimensions)
    }

    pub fn get_type(&self) -> VariableType {
        self.variable_type
    }

    pub fn is_f32(&self) -> bool {
        self.variable_type == VariableType::F32
    }

    pub fn is_i32(&self) -> bool {
        self.variable_type == VariableType::I32
    }

    pub fn borrow_dimensions(&self) -> &Vec<u64> {
        &self.dimensions
    }

    pub fn get_physical_size(&self) -> u64 {
        let mut size = match self.variable_type {
            VariableType::F32 | VariableType::I32 => 4,
            VariableType::B8 => 1
        };
        for dim in self.dimensions.iter() {
            size *= *dim;
        }
        size
    }
}
