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

    pub fn to_array_type(self, sizes: Vec<Expression>) -> DataType {
        DataType::array(self, sizes)
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
pub struct DataType {
    base: BaseType,
    sizes: Vec<Expression>,
}

impl DataType {
    pub fn scalar(base: BaseType) -> DataType {
        DataType {
            base,
            sizes: Vec::new(),
        }
    }

    pub fn new(base: BaseType) -> DataType {
        Self::scalar(base)
    }

    pub fn array(base: BaseType, sizes: Vec<Expression>) -> DataType {
        DataType { base, sizes }
    }

    // E.G. if size is 5, [2]Int becomes [5][2]Int.
    pub fn wrap_with_size(&mut self, size: Expression) {
        self.sizes.insert(0, size);
    }

    // E.G. if sizes is 5, 4, 3, then [2][2]Int becomes [5][4][3][2][2]Int.
    pub fn wrap_with_sizes(&mut self, mut sizes: Vec<Expression>) {
        sizes.append(&mut self.sizes);
        self.sizes = sizes;
    }

    // [2][3]Int being unwrapped once results in [3]Int
    // [4][2][3]Int being unwrapped twice results in [3]Int
    pub fn clone_and_unwrap(&self, num_unwraps: usize) -> DataType {
        DataType {
            base: self.base.clone(),
            sizes: Vec::from(&self.sizes[num_unwraps..]),
        }
    }

    pub fn borrow_sizes(&self) -> &Vec<Expression> {
        &self.sizes
    }

    pub fn set_literal_sizes(&mut self, literal_sizes: Vec<i64>) {
        assert!(literal_sizes.len() == self.sizes.len());
        for (index, size) in literal_sizes.into_iter().enumerate() {
            assert!(size > 0);
            self.sizes[index] =
                Expression::Literal(KnownData::Int(size), self.sizes[index].clone_position());
        }
    }

    pub fn borrow_base(&self) -> &BaseType {
        &self.base
    }

    pub fn is_scalar(&self) -> bool {
        self.sizes.len() == 0
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
        // Check if all the sizes have known and identical values.
        for (size, other_size) in self.sizes.iter().zip(other.sizes.iter()) {
            let size_value = program.borrow_value_of(size);
            let other_size_value = program.borrow_value_of(other_size);
            if let KnownData::Unknown = size_value {
                return false;
            } else if size_value != other_size_value {
                return false;
            }
        }
        true
    }
}

impl Debug for DataType {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        for size in self.sizes.iter() {
            if let Expression::Literal(KnownData::Int(dimension), ..) = size {
                write!(formatter, "[{}]", dimension)?;
            } else {
                write!(formatter, "[?]")?;
            }
        }
        write!(formatter, "{:?}", self.base)
    }
}