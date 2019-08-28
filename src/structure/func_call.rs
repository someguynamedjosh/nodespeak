use crate::structure::VariableId;

#[derive(Clone, Debug)]
pub struct VarAccess {
    base: VariableId,
    indexes: Vec<VariableId>,
}

impl VarAccess {
    pub fn new(base: VariableId) -> VarAccess {
        VarAccess {
            base: base,
            indexes: Vec::new(),
        }
    }

    pub fn add_index(&mut self, index: VariableId) {
        self.indexes.push(index);
    }

    pub fn get_base(&self) -> VariableId {
        self.base
    }

    pub fn iterate_over_indexes(&self) -> std::slice::Iter<VariableId> {
        self.indexes.iter()
    }
}

#[derive(Clone, Debug)]
pub struct FuncCall {
    function: VariableId,
    inputs: Vec<VarAccess>,
    outputs: Vec<VarAccess>,
}

impl FuncCall {
    pub fn new(function: VariableId) -> FuncCall {
        FuncCall {
            function: function,
            inputs: Vec::new(),
            outputs: Vec::new(),
        }
    }

    pub fn set_function(&mut self, function: VariableId) {
        self.function = function;
    }

    pub fn add_input(&mut self, input: VarAccess) {
        self.inputs.push(input);
    }

    pub fn add_output(&mut self, output: VarAccess) {
        self.outputs.push(output);
    }

    pub fn get_function(&self) -> VariableId {
        self.function
    }

    pub fn iterate_over_inputs(&self) -> std::slice::Iter<VarAccess> {
        self.inputs.iter()
    }

    pub fn iterate_over_outputs(&self) -> std::slice::Iter<VarAccess> {
        self.outputs.iter()
    }
}
