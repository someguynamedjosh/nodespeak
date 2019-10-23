use crate::trivial::structure::{Instruction, Variable};

use std::fmt::{self, Debug, Formatter};

#[derive(Clone, Copy)]
pub struct VariableId(usize);

impl Debug for VariableId {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        write!(formatter, "t{}", self.0)
    }
}

#[derive(Clone, Copy)]
pub struct LabelId(usize);

impl Debug for LabelId {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        write!(formatter, "l{}", self.0)
    }
}

pub struct Program {
    instructions: Vec<Instruction>,
    variables: Vec<Variable>,
    labels: usize,
}

impl Debug for Program {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        writeln!(formatter, "variables:")?;
        for variable in self.variables.iter() {
            writeln!(formatter, "  {:?}", variable)?;
        }
        writeln!(formatter, "{} labels", self.labels)?;
        writeln!(formatter, "instructions:")?;
        for instruction in self.instructions.iter() {
            writeln!(formatter, "  {:?}", instruction)?;
        }
        write!(formatter, "")
    }
}

impl Program {
    pub fn new() -> Program {
        Program {
            instructions: Vec::new(),
            variables: Vec::new(),
            labels: 0,
        }
    }

    pub fn add_instruction(&mut self, instruction: Instruction) {
        self.instructions.push(instruction);
    }

    pub fn borrow_instructions(&self) -> &Vec<Instruction> {
        &self.instructions
    }

    pub fn adopt_variable(&mut self, variable: Variable) -> VariableId {
        let id = VariableId(self.variables.len());
        self.variables.push(variable);
        id
    }

    pub fn borrow_variable(&self, id: VariableId) -> &Variable {
        &self.variables[id.0]
    }

    pub fn borrow_variable_mut(&mut self, id: VariableId) -> &mut Variable {
        &mut self.variables[id.0]
    }

    pub fn create_label(&mut self) -> LabelId {
        let id = LabelId(self.labels);
        self.labels += 1;
        id
    }
}
