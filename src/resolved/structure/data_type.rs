use std::fmt::{self, Debug, Formatter};

#[derive(Clone, PartialEq)]
pub enum BaseType {
    Bool,
    Int,
    Float,
}

impl BaseType {
    pub fn equivalent(&self, other: &BaseType) -> bool {
        self == other
    }

    pub fn to_scalar_type(self) -> DataType {
        DataType::scalar(self)
    }

    pub fn to_array_type(self, sizes: Vec<u64>) -> DataType {
        DataType::array(self, sizes)
    }
}

impl Debug for BaseType {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        match self {
            BaseType::Bool => write!(formatter, "Bool"),
            BaseType::Int => write!(formatter, "Int"),
            BaseType::Float => write!(formatter, "Float"),
        }
    }
}

#[derive(Clone, PartialEq)]
pub struct DataType {
    base: BaseType,
    dimensions: Vec<u64>,
}

impl DataType {
    pub fn scalar(base: BaseType) -> DataType {
        DataType {
            base,
            dimensions: Vec::new(),
        }
    }

    pub fn new(base: BaseType) -> DataType {
        Self::scalar(base)
    }

    pub fn array(base: BaseType, dimensions: Vec<u64>) -> DataType {
        DataType { base, dimensions }
    }

    // E.G. if dimension is 5, [2]Int becomes [5][2]Int.
    pub fn wrap_with_dimension(&mut self, dimension: u64) {
        self.dimensions.insert(0, dimension);
    }

    // E.G. if dimensions is 5, 4, 3, then [2][2]Int becomes [5][4][3][2][2]Int.
    pub fn wrap_with_dimensions(&mut self, mut dimensions: Vec<u64>) {
        dimensions.append(&mut self.dimensions);
        self.dimensions = dimensions;
    }

    // [2][3]Int being unwrapped once results in [3]Int
    // [4][2][3]Int being unwrapped twice results in [3]Int
    pub fn clone_and_unwrap(&self, num_unwraps: usize) -> DataType {
        println!("{:?} {:?}", self, self.dimensions);
        DataType {
            base: self.base.clone(),
            dimensions: Vec::from(&self.dimensions[num_unwraps..]),
        }
    }

    pub fn borrow_dimensions(&self) -> &Vec<u64> {
        &self.dimensions
    }

    pub fn set_dimensions(&mut self, new_dimensions: Vec<u64>) {
        assert!(new_dimensions.len() == self.dimensions.len());
        for (index, dimension) in new_dimensions.into_iter().enumerate() {
            self.dimensions[index] = dimension;
        }
    }

    pub fn borrow_base(&self) -> &BaseType {
        &self.base
    }

    pub fn is_scalar(&self) -> bool {
        self.dimensions.len() == 0
    }

    pub fn is_specific_scalar(&self, base_type: &BaseType) -> bool {
        self.is_scalar() && &self.base == base_type
    }

    pub fn is_array(&self) -> bool {
        !self.is_scalar()
    }

    pub fn equivalent(&self, other: &Self) -> bool {
        // Check if the base types are equivalent.
        if !self.base.equivalent(&other.base) {
            return false;
        }
        if self.dimensions.len() != other.dimensions.len() {
            return false;
        }
        for (dimension, other_dimension) in self.dimensions.iter().zip(other.dimensions.iter()) {
            if dimension != other_dimension {
                return false;
            }
        }
        true
    }
}

impl Debug for DataType {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        for dimension in self.dimensions.iter() {
            write!(formatter, "[{}]", dimension)?;
        }
        write!(formatter, "{:?}", self.base)
    }
}
