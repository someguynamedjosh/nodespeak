use super::{BaseType, DataType, FunctionData, KnownData};
use crate::problem::FilePosition;

use std::fmt::{self, Debug, Formatter};

#[derive(Clone)]
pub struct Variable {
    definition: FilePosition,

    data_type: DataType,
    initial_value: KnownData,
}

impl Debug for Variable {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        write!(formatter, "data type: {:?}", self.data_type)?;
        write!(formatter, "\ninitial value: {:?}", self.initial_value)
    }
}

// TODO: Read-only variables.
impl Variable {
    fn new_impl(
        definition: FilePosition,
        data_type: DataType,
        initial_value: Option<KnownData>,
    ) -> Variable {
        let initial_value = initial_value.unwrap_or_else(|| KnownData::Unknown);
        Variable {
            definition,
            initial_value: initial_value.clone(),
            data_type,
        }
    }

    pub fn variable(
        definition: FilePosition,
        data_type: DataType,
        initial_value: Option<KnownData>,
    ) -> Variable {
        Self::new_impl(definition, data_type, initial_value)
    }

    pub fn with_value(definition: FilePosition, data_type: DataType, value: KnownData) -> Variable {
        Self::new_impl(definition, data_type, Option::Some(value))
    }

    pub fn function_def(function_data: FunctionData) -> Variable {
        Self::with_value(
            function_data.get_header().clone(),
            BaseType::Function_.to_scalar_type(),
            KnownData::Function(function_data),
        )
    }

    pub fn data_type(definition: FilePosition, value: DataType) -> Variable {
        Self::with_value(
            definition,
            BaseType::DataType_.to_scalar_type(),
            KnownData::DataType(value),
        )
    }

    pub fn automatic(definition: FilePosition) -> Variable {
        Variable::variable(
            definition,
            BaseType::Automatic.to_scalar_type(),
            Option::None,
        )
    }

    pub fn void(definition: FilePosition) -> Variable {
        Variable::variable(definition, BaseType::Void.to_scalar_type(), Option::None)
    }

    pub fn set_definition(&mut self, new_definition: FilePosition) {
        self.definition = new_definition;
    }

    pub fn get_definition(&self) -> &FilePosition {
        &self.definition
    }

    pub fn set_data_type(&mut self, data_type: DataType) {
        self.data_type = data_type;
    }

    pub fn borrow_data_type(&self) -> &DataType {
        &self.data_type
    }

    pub fn borrow_data_type_mut(&mut self) -> &mut DataType {
        &mut self.data_type
    }

    pub fn set_initial_value(&mut self, value: KnownData) {
        self.initial_value = value;
    }

    pub fn borrow_initial_value(&self) -> &KnownData {
        &self.initial_value
    }
}
