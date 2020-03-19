use crate::shared::{NativeVar, LabelCounter, LabelId};
use crate::trivial::structure::Instruction;

use std::fmt::{self, Debug, Formatter};
use std::ops::{Index, IndexMut};

#[derive(Clone, Copy, Eq, Hash, PartialEq)]
pub struct VariableId(usize);

impl Debug for VariableId {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        write!(formatter, "tv{}", self.0)
    }
}

pub struct Program {
    instructions: Vec<Instruction>,
    variables: Vec<NativeVar>,
    inputs: Vec<VariableId>,
    outputs: Vec<VariableId>,
    label_counter: LabelCounter,
}

impl Debug for Program {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        writeln!(formatter, "variables:")?;
        for (index, variable) in self.variables.iter().enumerate() {
            writeln!(formatter, "  tv{}: {:?}", index, variable)?;
        }
        writeln!(formatter, "inputs:")?;
        for variable in self.inputs.iter() {
            writeln!(formatter, "  {:?}", variable)?;
        }
        writeln!(formatter, "outputs:")?;
        for variable in self.outputs.iter() {
            writeln!(formatter, "  {:?}", variable)?;
        }
        writeln!(formatter, "{:?}", self.label_counter)?;
        writeln!(formatter, "instructions:")?;
        for instruction in self.instructions.iter() {
            writeln!(formatter, "  {:?}", instruction)?;
        }
        write!(formatter, "")
    }
}

impl Index<VariableId> for Program {
    type Output = NativeVar;

    fn index(&self, variable: VariableId) -> &Self::Output {
        &self.variables[variable.0]
    }
}

impl IndexMut<VariableId> for Program {
    fn index_mut(&mut self, variable: VariableId) -> &mut Self::Output {
        &mut self.variables[variable.0]
    }
}

impl Program {
    pub fn new() -> Program {
        Program {
            instructions: Vec::new(),
            variables: Vec::new(),
            inputs: Vec::new(),
            outputs: Vec::new(),
            label_counter: LabelCounter::new(),
        }
    }

    pub fn add_instruction(&mut self, instruction: Instruction) {
        self.instructions.push(instruction);
    }

    pub fn borrow_instructions(&self) -> &Vec<Instruction> {
        &self.instructions
    }

    pub fn adopt_variable(&mut self, variable: NativeVar) -> VariableId {
        let id = VariableId(self.variables.len());
        self.variables.push(variable);
        id
    }

    pub fn borrow_variable(&self, id: VariableId) -> &NativeVar {
        &self.variables[id.0]
    }

    pub fn borrow_variable_mut(&mut self, id: VariableId) -> &mut NativeVar {
        &mut self.variables[id.0]
    }

    pub fn add_input(&mut self, input: VariableId) {
        self.inputs.push(input);
    }

    pub fn add_output(&mut self, output: VariableId) {
        self.outputs.push(output);
    }

    pub fn borrow_inputs(&self) -> &Vec<VariableId> {
        &self.inputs
    }

    pub fn borrow_outputs(&self) -> &Vec<VariableId> {
        &self.outputs
    }

    pub fn create_label(&mut self) -> LabelId {
        self.label_counter.create_label()
    }
}
