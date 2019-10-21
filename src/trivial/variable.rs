#[derive(Clone, Copy, Debug, PartialEq)]
pub enum VariableType {
    F32,
    I32,
}

#[derive(Clone, Debug)]
pub struct Variable {
    variable_type: VariableType,
    dimensions: Vec<usize>,
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

    pub fn new_array(typ: VariableType, dimensions: Vec<usize>) -> Variable {
        Variable {
            variable_type: typ,
            dimensions
        }
    }

    pub fn f32_array(dimensions: Vec<usize>) -> Variable {
        Self::new_array(VariableType::F32, dimensions)
    }

    pub fn i32_array(dimensions: Vec<usize>) -> Variable {
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

    pub fn borrow_dimensions(&self) -> &Vec<usize> {
        &self.dimensions
    }

    pub fn get_physical_size(&self) -> usize {
        let mut size = 1;
        for dim in self.dimensions.iter() {
            size *= *dim;
        }
        size
    }
}