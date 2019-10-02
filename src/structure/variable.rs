use super::{BaseType, DataType, FunctionData, KnownData};
use crate::problem::FilePosition;

use std::fmt::{self, Debug, Formatter};

#[derive(Clone)]
pub struct Variable {
    definition: FilePosition,

    data_type: DataType,
    initial_value: KnownData,
    permanent: bool,

    temporary_value: KnownData,
    // temporary_read: bool,
    // multiple_temporary_values: bool,
}

impl Debug for Variable {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        write!(formatter, "data type: {:?}", self.data_type)?;
        write!(formatter, "\ninitial value: {:?}", self.initial_value)
    }
}

impl Variable {
    fn new_impl(
        definition: FilePosition,
        data_type: DataType,
        initial_value: Option<KnownData>,
        permanent: bool,
    ) -> Variable {
        let initial_value = initial_value.unwrap_or_else(|| KnownData::Unknown);
        Variable {
            definition,
            initial_value: initial_value.clone(),
            data_type,
            permanent,
            temporary_value: initial_value, // temporary_read: false,
                                            // multiple_temporary_values: false,
        }
    }

    pub fn variable(
        definition: FilePosition,
        data_type: DataType,
        initial_value: Option<KnownData>,
    ) -> Variable {
        Self::new_impl(definition, data_type, initial_value, false)
    }

    pub fn constant(definition: FilePosition, data_type: DataType, value: KnownData) -> Variable {
        Self::new_impl(definition, data_type, Option::Some(value), true)
    }

    pub fn function_def(function_data: FunctionData) -> Variable {
        Self::constant(
            function_data.get_header().clone(),
            BaseType::Function_.to_scalar_type(),
            KnownData::Function(function_data),
        )
    }

    pub fn data_type(definition: FilePosition, value: DataType) -> Variable {
        Self::constant(
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

    pub fn bool_literal(definition: FilePosition, value: bool) -> Variable {
        Variable::constant(
            definition,
            BaseType::Bool.to_scalar_type(),
            KnownData::Bool(value),
        )
    }

    pub fn int_literal(definition: FilePosition, value: i64) -> Variable {
        Variable::constant(
            definition,
            BaseType::Int.to_scalar_type(),
            KnownData::Int(value),
        )
    }

    pub fn float_literal(definition: FilePosition, value: f64) -> Variable {
        Variable::constant(
            definition,
            BaseType::Float.to_scalar_type(),
            KnownData::Float(value),
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

    pub fn mark_as_permanent(&mut self) {
        self.permanent = true;
    }

    pub fn is_permanent(&self) -> bool {
        self.permanent
    }

    pub fn set_temporary_value(&mut self, value: KnownData) {
        self.temporary_value = value;
    }

    pub fn borrow_temporary_value(&self) -> &KnownData {
        &self.temporary_value
    }

    pub fn borrow_temporary_value_mut(&mut self) -> &mut KnownData {
        &mut self.temporary_value
    }

    pub fn reset_temporary_value(&mut self) {
        self.temporary_value = self.initial_value.clone();
    }

    // pub fn set_temporary_value(&mut self, value: KnownData) {
    //     self.temporary_value = value;
    //     if self.temporary_read {
    //         self.multiple_temporary_values = true;
    //     }
    // }

    // pub fn borrow_temporary_value(&mut self) -> &KnownData {
    //     self.temporary_read = true;
    //     &self.temporary_value
    // }

    // pub fn reset_temporary_value(&mut self) {
    //     self.temporary_value = self.initial_value.clone();
    //     self.temporary_read = false;
    //     self.multiple_temporary_values = false;
    // }

    // pub fn had_multiple_temporary_values(&self) -> bool {
    //     self.multiple_temporary_values
    // }
}
