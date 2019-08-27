use crate::structure::{EntityId, Program, ScopeId};

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

#[derive(Clone, Debug)]
pub struct FunctionEntity {
    inputs: Vec<EntityId>,
    outputs: Vec<EntityId>,
    body: ScopeId,
    // Put code into this scope to describe how template parameters should be set up and used.
    template_descriptor: Option<ScopeId>,
    // Variables or data types that are created inside the function's signature. This is necessary to distinguish template parameters from variables declared in the function body.
    template_parameters: Vec<EntityId>,
}

impl FunctionEntity {
    pub fn new(body: ScopeId) -> FunctionEntity {
        FunctionEntity {
            inputs: Vec::new(),
            outputs: Vec::new(),
            body: body,
            template_descriptor: Option::None,
            template_parameters: Vec::new(),
        }
    }

    pub fn add_template_parameter(&mut self, template_parameter: EntityId) {
        self.template_parameters.push(template_parameter);
    }

    pub fn iterate_over_template_parameters(&self) -> std::slice::Iter<EntityId> {
        self.template_parameters.iter()
    }

    pub fn get_template_descriptor(&mut self, program: &mut Program) -> ScopeId {
        if self.template_descriptor.is_none() {
            self.template_descriptor = Option::Some(program.create_child_scope(self.body));
        }
        self.template_descriptor.unwrap()
    }

    pub fn add_input(&mut self, input: EntityId) {
        self.inputs.push(input);
    }

    pub fn get_input(&self, index: usize) -> EntityId {
        return self.inputs[index];
    }

    pub fn get_num_inputs(&self) -> usize {
        return self.inputs.len();
    }

    pub fn iterate_over_inputs(&self) -> std::slice::Iter<EntityId> {
        self.inputs.iter()
    }

    pub fn add_output(&mut self, output: EntityId) {
        self.outputs.push(output);
    }

    pub fn get_output(&self, index: usize) -> EntityId {
        return self.outputs[index];
    }

    pub fn get_num_outputs(&self) -> usize {
        return self.outputs.len();
    }

    pub fn iterate_over_outputs(&self) -> std::slice::Iter<EntityId> {
        self.outputs.iter()
    }

    pub fn get_single_output(&self) -> Option<EntityId> {
        if self.outputs.len() == 1 {
            Option::Some(self.outputs[0])
        } else {
            Option::None
        }
    }

    pub fn get_body(&self) -> ScopeId {
        self.body
    }
}

#[derive(Clone, Debug)]
pub enum BuiltinFunction {
    Add,
    Subtract,
    Multiply,
    Divide,
    IntDiv,
    Modulo,
    Power,
    Reciprocal,

    BAnd,
    BOr,
    BXor,
    BNot,

    IntToFloat,
    FloatToInt,
    IntToBool,
    BoolToInt,
    FloatToBool,
    BoolToFloat,

    Equal,
    NotEqual,
    LessThanOrEqual,
    LessThan,
    GreaterThanOrEqual,
    GreaterThan,

    And,
    Or,
    Xor,
    Not,

    Assert,
    Copy,
    Return,
}

#[derive(Debug)]
pub struct BuiltinFunctionEntity {
    base: FunctionEntity,
    func: BuiltinFunction,
}

impl BuiltinFunctionEntity {
    pub fn new(func: BuiltinFunction, program: &mut Program) -> BuiltinFunctionEntity {
        let scope = program.create_child_scope(program.get_root_scope());
        BuiltinFunctionEntity {
            base: FunctionEntity::new(scope),
            func: func,
        }
    }

    pub fn add_template_parameter(&mut self, template_parameter: EntityId) {
        self.base.add_template_parameter(template_parameter);
    }

    pub fn get_template_descriptor(&mut self, program: &mut Program) -> ScopeId {
        self.base.get_template_descriptor(program)
    }

    pub fn add_input(&mut self, input: EntityId) {
        self.base.add_input(input);
    }

    pub fn add_output(&mut self, output: EntityId) {
        self.base.add_output(output);
    }

    pub fn get_single_output(&self) -> Option<EntityId> {
        self.base.get_single_output()
    }

    pub fn get_func(&self) -> BuiltinFunction {
        self.func.clone()
    }

    pub fn get_base(&self) -> FunctionEntity {
        self.base.clone()
    }

    pub fn get_scope(&self) -> ScopeId {
        self.base.body
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ArrayDataType {
    base: DataType,
    size: EntityId,
}

#[derive(Clone, Debug, PartialEq)]
pub enum DataType {
    Automatic,
    AwaitingTemplate,
    Bool,
    Int,
    Float,
    Void,
    DataType_,
    Function_,
    Array(Box<ArrayDataType>),
}

impl DataType {
    pub fn is_automatic(&self) -> bool {
        match self {
            DataType::Automatic => true,
            DataType::Array(data) => data.base.is_automatic(),
            _ => false,
        }
    }

    pub fn is_awaiting_template(&self) -> bool {
        match self {
            DataType::AwaitingTemplate => true,
            DataType::Array(data) => data.base.is_awaiting_template(),
            _ => false,
        }
    }
}

#[derive(Debug)]
pub enum Entity {
    Variable(VariableEntity),
    Function(FunctionEntity),
    BuiltinFunction(BuiltinFunctionEntity),
    BoolLiteral(bool),
    IntLiteral(i64),
    FloatLiteral(f64),
    DataType(DataType),
}

pub fn make_var(data_type: EntityId) -> Entity {
    Entity::Variable(VariableEntity::new(data_type))
}
