use crate::vague::{EntityId, ScopeId};

#[derive(Debug)]
pub struct VariableEntity {
    data_type: EntityId,
}

impl VariableEntity {
    pub fn new(data_type: EntityId) -> VariableEntity {
        VariableEntity {
            data_type: data_type,
        }
    }

    pub fn get_data_type(&self) -> EntityId {
        self.data_type
    }
}

#[derive(Debug)]
pub struct FunctionEntity {
    inputs: Vec<EntityId>,
    outputs: Vec<EntityId>,
    body: ScopeId,
}

impl FunctionEntity {
    pub fn new(body: ScopeId) -> FunctionEntity {
        FunctionEntity {
            inputs: Vec::new(),
            outputs: Vec::new(),
            body: body,
        }
    }

    pub fn add_input(&mut self, input: EntityId) {
        self.inputs.push(input);
    }

    pub fn add_output(&mut self, output: EntityId) {
        self.outputs.push(output);
    }

    pub fn get_single_output(&self) -> Option<EntityId> {
        if self.outputs.len() == 1 {
            Option::Some(self.inputs[0])
        } else {
            Option::None
        }
    }
}

#[derive(Debug)]
pub struct ArrayDataType {
    base: DataType,
    size: EntityId,
}

#[derive(Debug)]
pub enum DataType {
    Automatic,
    Int,
    Float,
    DataType_,
    Function_,
    Array(Box<ArrayDataType>),
}

#[derive(Debug)]
pub enum Entity {
    Variable(VariableEntity),
    Function(FunctionEntity),
    BuiltinFunction(String),
    IntLiteral(i64),
    FloatLiteral(f64),
    DataType(DataType),
}
