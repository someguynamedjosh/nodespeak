use crate::problem::FilePosition;
use crate::structure::{BuiltinFunction, ScopeId, VariableId};

#[derive(Clone, Debug, PartialEq)]
pub enum DataType {
    Automatic,
    Dynamic(VariableId),
    LoadTemplateParameter(VariableId),
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
pub struct FunctionData {
    body: ScopeId,
    builtin: Option<BuiltinFunction>,
    header: FilePosition,
    inputs: Vec<VariableId>,
    outputs: Vec<VariableId>,
}

impl FunctionData {
    pub fn new(body: ScopeId, header: FilePosition) -> FunctionData {
        FunctionData {
            body,
            builtin: None,
            header,
            inputs: Vec::new(),
            outputs: Vec::new(),
        }
    }

    pub fn builtin(body: ScopeId, builtin: BuiltinFunction, header: FilePosition) -> FunctionData {
        FunctionData {
            body,
            builtin: Option::Some(builtin),
            header,
            inputs: Vec::new(),
            outputs: Vec::new(),
        }
    }

    pub fn set_header(&mut self, new_header: FilePosition) {
        self.header = new_header;
    }

    pub fn get_header(&self) -> &FilePosition {
        &self.header
    }

    pub fn get_body(&self) -> ScopeId {
        self.body
    }

    pub fn is_builtin(&self) -> bool {
        self.builtin.is_some()
    }

    pub fn get_builtin(&self) -> &Option<BuiltinFunction> {
        &self.builtin
    }

    pub fn add_input(&mut self, input: VariableId) {
        self.inputs.push(input)
    }

    pub fn get_input(&self, index: usize) -> VariableId {
        self.inputs[index]
    }

    pub fn borrow_inputs(&self) -> &Vec<VariableId> {
        &self.inputs
    }

    pub fn add_output(&mut self, output: VariableId) {
        self.outputs.push(output)
    }

    pub fn get_output(&self, index: usize) -> VariableId {
        self.outputs[index]
    }

    pub fn borrow_outputs(&self) -> &Vec<VariableId> {
        &self.outputs
    }

    pub fn get_single_output(&self) -> Option<VariableId> {
        if self.outputs.len() == 1 {
            Option::Some(self.outputs[0].clone())
        } else {
            None
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
    Function(FunctionData),
    // Array(Vec<KnownData>),
}

#[derive(Clone, Debug)]
pub struct Variable {
    definition: FilePosition,

    data_type: DataType,
    initial_value: KnownData,
    permanent: bool,

    temporary_value: KnownData,
    temporary_read: bool,
    multiple_temporary_values: bool,
}

impl Variable {
    fn new_impl(
        definition: FilePosition,
        data_type: DataType,
        initial_value: Option<KnownData>,
        permanent: bool,
    ) -> Variable {
        Variable {
            definition,
            initial_value: initial_value.unwrap_or_else(|| KnownData::Unknown),
            data_type,
            permanent,
            temporary_value: KnownData::Unknown,
            temporary_read: false,
            multiple_temporary_values: false,
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
            DataType::Function_,
            KnownData::Function(function_data),
        )
    }

    pub fn data_type(definition: FilePosition, value: DataType) -> Variable {
        Self::constant(definition, DataType::DataType_, KnownData::DataType(value))
    }

    pub fn automatic(definition: FilePosition) -> Variable {
        Variable::variable(definition, DataType::Automatic, Option::None)
    }

    pub fn bool_literal(definition: FilePosition, value: bool) -> Variable {
        Variable::constant(definition, DataType::Bool, KnownData::Bool(value))
    }

    pub fn int_literal(definition: FilePosition, value: i64) -> Variable {
        Variable::constant(definition, DataType::Int, KnownData::Int(value))
    }

    pub fn float_literal(definition: FilePosition, value: f64) -> Variable {
        Variable::constant(definition, DataType::Float, KnownData::Float(value))
    }

    pub fn void(definition: FilePosition) -> Variable {
        Variable::variable(definition, DataType::Void, Option::None)
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

    pub fn borrow_temporary_value(&mut self) -> &KnownData {
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
