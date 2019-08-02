use crate::vague::EntityId;

#[derive(Debug)]
pub struct VarAccess {
    base: EntityId,
    indexes: Vec<EntityId>,
}

impl VarAccess {
    pub fn new(base: EntityId) -> VarAccess {
        VarAccess {
            base: base,
            indexes: Vec::new()
        }
    }

    pub fn add_index(&mut self, index: EntityId) {
        self.indexes.push(index);
    }
}

#[derive(Debug)]
pub struct FuncCall {
    function: EntityId,
    inputs: Vec<VarAccess>,
    outputs: Vec<VarAccess>,
}

impl FuncCall {
    pub fn new(function: EntityId) -> FuncCall {
        FuncCall {
            function: function,
            inputs: Vec::new(),
            outputs: Vec::new(),
        }
    }

    pub fn add_input(&mut self, input: VarAccess) {
        self.inputs.push(input);
    }

    pub fn add_output(&mut self, output: VarAccess) {
        self.outputs.push(output);
    }
}