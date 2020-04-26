use std::fmt::{self, Debug, Formatter};

#[derive(Clone, Copy, PartialEq)]
pub enum BaseType {
    B1,
    I32,
    F32,
}

impl Debug for BaseType {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        write!(
            formatter,
            "{}",
            match self {
                Self::B1 => "b1",
                Self::I32 => "i32",
                Self::F32 => "f32",
            }
        )
    }
}

#[derive(Clone, PartialEq)]
pub struct DataType {
    base: BaseType,
    dimensions: Vec<usize>,
}

impl Debug for DataType {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        for dim in self.dimensions.iter() {
            write!(formatter, "[{}]", dim)?;
        }
        write!(formatter, "{:?}", self.base)
    }
}

impl DataType {
    pub fn array(base: BaseType, dimensions: Vec<usize>) -> DataType {
        DataType { base, dimensions }
    }

    pub fn scalar(base: BaseType) -> DataType {
        Self::array(base, vec![])
    }

    pub fn i32_scalar() -> DataType {
        Self::scalar(BaseType::I32)
    }

    pub fn f32_scalar() -> DataType {
        Self::scalar(BaseType::F32)
    }

    pub fn b1_scalar() -> DataType {
        Self::scalar(BaseType::B1)
    }

    pub fn get_base(&self) -> BaseType {
        self.base
    }

    pub fn borrow_dimensions(&self) -> &Vec<usize> {
        &self.dimensions
    }

    pub fn is_int(&self) -> bool {
        self.base == BaseType::I32
    }

    pub fn is_float(&self) -> bool {
        self.base == BaseType::F32
    }

    pub fn is_scalar(&self) -> bool {
        self.dimensions.len() == 0
    }

    pub fn is_array(&self) -> bool {
        self.dimensions.len() > 0
    }

    // [2][3]Int being unwrapped once results in [3]Int
    // [4][2][3]Int being unwrapped twice results in [3]Int
    pub fn clone_and_unwrap(&self, num_dimensions: usize) -> Self {
        Self {
            base: self.base.clone(),
            dimensions: (&self.dimensions[num_dimensions..]).into(),
        }
    }

    pub fn wrap_with_dimension(&self, dimension: usize) -> Self {
        Self {
            base: self.base,
            dimensions: [dimension]
                .iter()
                .chain(self.dimensions.iter())
                .cloned()
                .collect(),
        }
    }
}

#[derive(Clone)]
pub struct Variable {
    typ: DataType,
}

impl Debug for Variable {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        write!(formatter, "{:?}", self.typ)
    }
}

impl Variable {
    pub fn new(typ: DataType) -> Variable {
        Variable { typ }
    }

    pub fn borrow_type(&self) -> &DataType {
        &self.typ
    }

    pub fn borrow_dimensions(&self) -> &Vec<usize> {
        self.typ.borrow_dimensions()
    }
}
