use crate::structure::{ScopeId, VariableId};

#[derive(Clone, Debug, PartialEq)]
pub enum DataType {
    Automatic,
    Dynamic(VariableId),
    Bool,
    Int,
    Float,
    Void,
    DataType_,
    Function_,
    // Array(Box<ArrayDataType>),
}

impl DataType {
    pub fn is_automatic(&self) -> bool {
        match self {
            DataType::Automatic => true,
            // DataType::Array(data) => data.base.is_automatic(),
            _ => false,
        }
    }
}

#[derive(Clone, Debug)]
pub enum KnownData {
    Void,
    Unknown,
    Bool(bool),
    Int(i64),
    Float(f64),
    DataType(DataType),
    Function {
        body: ScopeId,
        inputs: Vec<VariableId>,
        outputs: Vec<VariableId>,
    }, // Array(Vec<KnownData>),
}

impl KnownData {
    pub fn default_for_type(dtype: &DataType) -> KnownData {
        match dtype {
            DataType::Automatic => KnownData::Unknown,
            DataType::Dynamic(_source) => KnownData::Unknown,
            DataType::Bool => KnownData::Bool(false),
            DataType::Int => KnownData::Int(0),
            DataType::Float => KnownData::Float(0.0),
            DataType::DataType_ => KnownData::Unknown,
            DataType::Function_ => KnownData::Unknown,
        }
    }
}

#[derive(Debug)]
pub struct Variable {
    data_type: DataType,
    initial_value: KnownData,
    permanent: bool,

    temporary_value: KnownData,
    temporary_read: bool,
    multiple_temporary_values: bool,
}

impl Variable {
    fn new_impl(
        data_type: DataType,
        initial_value: Option<KnownData>,
        permanent: bool,
    ) -> Variable {
        Variable {
            data_type,
            initial_value: initial_value.unwrap_or_else(|| KnownData::default_for_type(&data_type)),
            permanent,
            temporary_value: KnownData::Unknown,
            temporary_read: false,
            multiple_temporary_values: false,
        }
    }

    pub fn variable(data_type: DataType, initial_value: Option<KnownData>) -> Variable {
        Self::new_impl(data_type, initial_value, false)
    }

    pub fn constant(data_type: DataType, value: KnownData) -> Variable {
        Self::new_impl(data_type, Option::Some(value), true)
    }

    pub fn automatic() -> Variable {
        Variable::variable(DataType::Automatic, Option::None)
    }

    pub fn bool_literal(value: bool) -> Variable {
        Variable::constant(DataType::Bool, KnownData::Bool(value))
    }

    pub fn int_literal(value: i64) -> Variable {
        Variable::constant(DataType::Int, KnownData::Int(value))
    }

    pub fn float_literal(value: f64) -> Variable {
        Variable::constant(DataType::Float, KnownData::Float(value))
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
        if self.temporary_read {
            self.multiple_temporary_values = true;
        }
    }

    pub fn borrow_temporary_value(&self) -> &KnownData {
        self.temporary_read = true;
        &self.temporary_value
    }

    pub fn reset_temporary_value(&mut self) {
        self.temporary_value = self.initial_value.clone();
        self.temporary_read = false;
        self.multiple_temporary_values = false;
    }

    pub fn had_multiple_temporary_values(&self) -> bool {
        self.multiple_temporary_values
    }
}
