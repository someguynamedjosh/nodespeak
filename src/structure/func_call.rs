use crate::problem::FilePosition;
use crate::structure::{DataType, Program, VariableId};

#[derive(Clone, Debug)]
pub struct VarAccess {
    position: FilePosition,
    base: VariableId,
    indexes: Vec<VarAccess>,
}

impl PartialEq for VarAccess {
    fn eq(&self, other: &Self) -> bool {
        self.base == other.base && self.indexes == other.indexes
    }
}

impl VarAccess {
    pub fn new(position: FilePosition, base: VariableId) -> VarAccess {
        VarAccess {
            position,
            base: base,
            indexes: Vec::new(),
        }
    }

    pub fn set_position(&mut self, new_position: FilePosition) {
        self.position = new_position;
    }

    pub fn get_position(&self) -> &FilePosition {
        &self.position
    }

    pub fn add_index(&mut self, index: VarAccess) {
        self.indexes.push(index);
    }

    pub fn borrow_indexes(&self) -> &Vec<VarAccess> {
        &self.indexes
    }

    pub fn get_base(&self) -> VariableId {
        self.base
    }

    pub fn iterate_over_indexes(&self) -> std::slice::Iter<VarAccess> {
        self.indexes.iter()
    }

    pub fn borrow_data_type<'a>(&'a self, program: &'a Program) -> &'a DataType {
        let data_type = program.borrow_variable(self.base).borrow_data_type();
        // For now, we don't have any code to manage arrays because arrays didn't exist at the time
        // of writing this code.
        assert!(self.indexes.len() == 0);
        data_type
    }

    pub fn with_additional_index(&self, index: VarAccess) -> VarAccess {
        let mut result = self.clone();
        result.add_index(index);
        result
    }
}

#[derive(Clone, Debug)]
pub struct FuncCall {
    function: VariableId,
    position: FilePosition,
    inputs: Vec<VarAccess>,
    outputs: Vec<VarAccess>,
}

impl FuncCall {
    pub fn new(function: VariableId, position: FilePosition) -> FuncCall {
        FuncCall {
            function: function,
            position,
            inputs: Vec::new(),
            outputs: Vec::new(),
        }
    }

    pub fn set_position(&mut self, new_position: FilePosition) {
        self.position = new_position;
    }

    pub fn get_position(&self) -> &FilePosition {
        &self.position
    }

    pub fn set_function(&mut self, function: VariableId) {
        self.function = function;
    }

    pub fn add_input(&mut self, input: VarAccess) {
        self.inputs.push(input);
    }

    pub fn borrow_inputs(&self) -> &Vec<VarAccess> {
        &self.inputs
    }

    pub fn add_output(&mut self, output: VarAccess) {
        self.outputs.push(output);
    }

    pub fn borrow_outputs(&self) -> &Vec<VarAccess> {
        &self.outputs
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
