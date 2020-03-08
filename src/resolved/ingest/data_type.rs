use std::fmt::{self, Debug, Formatter};

use super::{problems, CompileProblem, Content, ScopeSimplifier};
use crate::resolved::structure as o;
use crate::vague::structure as i;

#[derive(Clone, PartialEq)]
pub enum BaseType {
    Bool,
    Int,
    Float,
    Function_,
    DataType_,
    Void,
    Automatic,
}

impl BaseType {
    pub fn equivalent(&self, other: &BaseType) -> bool {
        self == other
    }

    pub fn to_scalar_type(self) -> DataType {
        DataType::scalar(self)
    }

    pub fn to_array_type(self, sizes: Vec<usize>) -> DataType {
        DataType::array(self, sizes)
    }
}

impl Debug for BaseType {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        match self {
            BaseType::Bool => write!(formatter, "Bool"),
            BaseType::Int => write!(formatter, "Int"),
            BaseType::Float => write!(formatter, "Float"),
            BaseType::Function_ => write!(formatter, "Function_"),
            BaseType::DataType_ => write!(formatter, "DataType_"),
            BaseType::Void => write!(formatter, "Void"),
            BaseType::Automatic => write!(formatter, "Automatic"),
        }
    }
}

#[derive(Clone, PartialEq)]
pub struct DataType {
    base: BaseType,
    dimensions: Vec<usize>,
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

    pub fn array(base: BaseType, dimensions: Vec<usize>) -> DataType {
        DataType { base, dimensions }
    }

    pub fn from_output_type(from: &o::DataType) -> DataType {
        Self::array(
            match from.borrow_base() {
                o::BaseType::Bool => BaseType::Bool,
                o::BaseType::Float => BaseType::Float,
                o::BaseType::Int => BaseType::Int,
            },
            from.borrow_dimensions()
                .iter()
                .map(|(size, _)| *size)
                .collect(),
        )
    }

    // E.G. if dimension is 5, [2]Int becomes [5][2]Int.
    pub fn wrap_with_dimension(&mut self, dimension: usize) {
        self.dimensions.insert(0, dimension);
    }

    // E.G. if dimensions is 5, 4, 3, then [2][2]Int becomes [5][4][3][2][2]Int.
    pub fn wrap_with_dimensions(&mut self, mut dimensions: Vec<usize>) {
        dimensions.append(&mut self.dimensions);
        self.dimensions = dimensions;
    }

    // [2][3]Int being unwrapped once results in [3]Int
    // [4][2][3]Int being unwrapped twice results in [3]Int
    pub fn clone_and_unwrap(&self, num_unwraps: usize) -> DataType {
        DataType {
            base: self.base.clone(),
            dimensions: (&self.dimensions[num_unwraps..]).into(),
        }
    }

    pub fn borrow_dimensions(&self) -> &[usize] {
        &self.dimensions
    }

    pub fn set_dimensions(&mut self, new_dimensions: Vec<usize>) {
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

    pub fn is_automatic(&self) -> bool {
        match self.base {
            BaseType::Automatic => true,
            _ => false,
        }
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

    pub fn to_output_type(&self) -> Result<o::DataType, ()> {
        let new_base = match self.base {
            BaseType::Bool => o::BaseType::Bool,
            BaseType::Int => o::BaseType::Int,
            BaseType::Float => o::BaseType::Float,
            BaseType::DataType_ | BaseType::Function_ | BaseType::Automatic | BaseType::Void => {
                return Result::Err(())
            }
        };
        Result::Ok(o::DataType::array(new_base, self.dimensions.clone()))
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

impl<'a> ScopeSimplifier<'a> {
    pub fn input_to_intermediate_type(
        &mut self,
        source: i::DataType,
    ) -> Result<DataType, CompileProblem> {
        let mut new_dimensions = vec![];
        for old_dimension in source.borrow_dimensions() {
            match self.resolve_expression(&old_dimension)?.content {
                Content::Interpreted(data) => match data {
                    i::KnownData::Int(value) => {
                        if value > 0 {
                            new_dimensions.push(value as usize);
                        } else {
                            return Err(problems::array_size_not_positive(
                                old_dimension.clone_position(),
                                value,
                            ));
                        }
                    }
                    _ => {
                        return Err(problems::array_size_not_int(
                            old_dimension.clone_position(),
                            &data.get_data_type().unwrap(),
                        ));
                    }
                },
                Content::Modified(..) => {
                    return Err(problems::array_size_not_resolved(
                        old_dimension.clone_position(),
                    ));
                }
            }
        }
        let base_type = match source.borrow_base() {
            i::BaseType::Automatic => BaseType::Automatic,
            i::BaseType::Dynamic(..) => unimplemented!("TODO: Dynamic types."),
            i::BaseType::LoadTemplateParameter(..) => unimplemented!("TODO: Template parameters."),
            i::BaseType::Bool => BaseType::Bool,
            i::BaseType::Int => BaseType::Int,
            i::BaseType::Float => BaseType::Float,
            i::BaseType::DataType_ => BaseType::DataType_,
            i::BaseType::Function_ => BaseType::Function_,
            i::BaseType::Void => BaseType::Void,
        };
        Result::Ok(DataType::array(base_type, new_dimensions))
    }
}
