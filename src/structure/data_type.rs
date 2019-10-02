use super::KnownData;

use crate::structure::{Expression, Program, VariableId};

use std::fmt::{self, Debug, Formatter};

#[derive(Clone, PartialEq)]
pub enum BaseType {
    Automatic,
    Dynamic(VariableId),
    LoadTemplateParameter(VariableId),
    Bool,
    Int,
    Float,
    Void,
    DataType_,
    Function_,
}

impl BaseType {
    pub fn is_automatic(&self) -> bool {
        match self {
            BaseType::Automatic => true,
            // BaseType::Array(data) => data.base.is_automatic(),
            _ => false,
        }
    }

    pub fn equivalent(&self, other: &BaseType, program: &Program) -> bool {
        match self {
            // If it's a basic type, just check if it is equal to the other one.
            BaseType::Automatic
            | BaseType::Bool
            | BaseType::Int
            | BaseType::Float
            | BaseType::Void
            | BaseType::DataType_
            | BaseType::Function_ => self == other,
            // If it's a dynamic / template type, check the value contained in both targets are both
            // known and identical.
            BaseType::Dynamic(target) | BaseType::LoadTemplateParameter(target) => match other {
                BaseType::Dynamic(other_target) | BaseType::LoadTemplateParameter(other_target) => {
                    let value = program.borrow_temporary_value(*target);
                    let other_value = program.borrow_temporary_value(*other_target);
                    if let KnownData::Unknown = value {
                        false
                    } else {
                        value == other_value
                    }
                }
                _ => false,
            },
        }
    }

    pub fn to_scalar_type(self) -> DataType {
        DataType::scalar(self)
    }

    pub fn to_literal_array_type(self, sizes: Vec<Expression>) -> DataType {
        DataType::array(
            self,
            sizes
                .into_iter()
                .map(|ex| ArrayDimension::literal(ex))
                .collect(),
        )
    }
}

impl Debug for BaseType {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        match self {
            BaseType::Automatic => write!(formatter, "Auto"),
            BaseType::Dynamic(_var) => write!(formatter, "Unresolved"),
            BaseType::LoadTemplateParameter(_var) => write!(formatter, "Unresolved"),
            BaseType::Bool => write!(formatter, "Bool"),
            BaseType::Int => write!(formatter, "Int"),
            BaseType::Float => write!(formatter, "Float"),
            BaseType::Void => write!(formatter, "Void"),
            BaseType::DataType_ => write!(formatter, "DataType_"),
            BaseType::Function_ => write!(formatter, "Function_"),
        }
    }
}

#[derive(Clone, PartialEq)]
pub enum ArrayDimensionType {
    /// The underlying data is an N length array. The index provided for the corresponding dimension
    /// should be used to select an element from the array. This is the default.
    Literal,
    /// The underlying data is a scalar value/ No matter what value is provided to index the
    /// corresponding dimension, that first item should always be returned.
    ScalarProxy,
    /// The underlying data is an array of 1 item. No matter what value is provided to index the
    /// corresponding dimension, that first item should always be returned.
    SingleElementProxy,
}

#[derive(Clone, PartialEq)]
pub struct ArrayDimension {
    pub size: Expression,
    pub typ: ArrayDimensionType,
}

impl ArrayDimension {
    pub fn literal(size: Expression) -> ArrayDimension {
        ArrayDimension {
            size,
            typ: ArrayDimensionType::Literal,
        }
    }

    pub fn scalar_proxy(size: Expression) -> ArrayDimension {
        ArrayDimension {
            size,
            typ: ArrayDimensionType::ScalarProxy,
        }
    }

    pub fn single_element_proxy(size: Expression) -> ArrayDimension {
        ArrayDimension {
            size,
            typ: ArrayDimensionType::SingleElementProxy,
        }
    }
}

#[derive(Clone, PartialEq)]
pub struct DataType {
    base: BaseType,
    dimensions: Vec<ArrayDimension>,
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

    pub fn array(base: BaseType, dimensions: Vec<ArrayDimension>) -> DataType {
        DataType { base, dimensions }
    }

    // E.G. if dimension is 5, [2]Int becomes [5][2]Int.
    pub fn wrap_with_dimension(&mut self, dimension: ArrayDimension) {
        self.dimensions.insert(0, dimension);
    }

    // E.G. if dimensions is 5, 4, 3, then [2][2]Int becomes [5][4][3][2][2]Int.
    pub fn wrap_with_dimensions(&mut self, mut dimensions: Vec<ArrayDimension>) {
        dimensions.append(&mut self.dimensions);
        self.dimensions = dimensions;
    }

    // [2][3]Int being unwrapped once results in [3]Int
    // [4][2][3]Int being unwrapped twice results in [3]Int
    pub fn clone_and_unwrap(&self, num_unwraps: usize) -> DataType {
        DataType {
            base: self.base.clone(),
            dimensions: Vec::from(&self.dimensions[num_unwraps..]),
        }
    }

    pub fn borrow_dimensions(&self) -> &Vec<ArrayDimension> {
        &self.dimensions
    }

    pub fn set_literal_dimensions(&mut self, literal_dimensions: Vec<i64>) {
        assert!(literal_dimensions.len() == self.dimensions.len());
        for (index, dimension) in literal_dimensions.into_iter().enumerate() {
            assert!(dimension > 0);
            self.dimensions[index].size = Expression::Literal(
                KnownData::Int(dimension),
                self.dimensions[index].size.clone_position(),
            );
        }
    }

    pub fn borrow_base(&self) -> &BaseType {
        &self.base
    }

    pub fn is_scalar(&self) -> bool {
        self.dimensions.len() == 0
    }

    pub fn is_array(&self) -> bool {
        !self.is_scalar()
    }

    pub fn is_automatic(&self) -> bool {
        self.base.is_automatic()
    }

    pub fn equivalent(&self, other: &Self, program: &Program) -> bool {
        // Check if the base types are equivalent.
        if !self.base.equivalent(&other.base, program) {
            return false;
        }
        // Check if all the dimensions have known and identical values.
        for (dimension, other_dimension) in self.dimensions.iter().zip(other.dimensions.iter()) {
            let dimension_value = program.borrow_value_of(&dimension.size);
            let other_dimension_value = program.borrow_value_of(&other_dimension.size);
            if let KnownData::Unknown = dimension_value {
                return false;
            } else if dimension_value != other_dimension_value {
                return false;
            }
        }
        true
    }
}

impl Debug for DataType {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        for dimension in self.dimensions.iter() {
            if let Expression::Literal(KnownData::Int(dimension), ..) = dimension.size {
                write!(formatter, "[{}]", dimension)?;
            } else {
                write!(formatter, "[?]")?;
            }
        }
        write!(formatter, "{:?}", self.base)
    }
}
