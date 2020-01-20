use super::Variable;
use std::fmt::{self, Debug, Formatter};
use std::ops::{Index, IndexMut};

#[derive(Clone, Copy, Eq, Hash, PartialEq)]
pub struct VariableId(usize);

impl Debug for VariableId {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        write!(formatter, "v{}", self.0)
    }
}

pub struct GenericProgram<Instruction> {
    variables: Vec<Variable>,
    instructions: Vec<Instruction>,
}

impl<Instruction> Index<VariableId> for GenericProgram<Instruction> {
    type Output = Variable;

    fn index(&self, variable: VariableId) -> &Self::Output {
        &self.variables[variable.0]
    }
}

impl<Instruction> IndexMut<VariableId> for GenericProgram<Instruction> {
    fn index_mut(&mut self, variable: VariableId) -> &mut Self::Output {
        &mut self.variables[variable.0]
    }
}

impl<Instruction> GenericProgram<Instruction> {
    pub fn new() -> GenericProgram<Instruction> {
        GenericProgram {
            variables: Vec::new(),
            instructions: Vec::new(),
        }
    }

    pub fn adopt_variable(&mut self, variable: Variable) -> VariableId {
        let id = VariableId(self.variables.len());
        self.variables.push(variable);
        id
    }

    pub fn add_instruction(&mut self, instruction: Instruction) {
        self.instructions.push(instruction);
    }

    pub fn borrow_instructions(&self) -> &Vec<Instruction> {
        &self.instructions
    }
}

impl<Instruction: Debug> Debug for GenericProgram<Instruction> {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        write!(formatter, "instructions:\n")?;
        for instruction in &self.instructions {
            write!(formatter, "{:?}\n", instruction)?;
        }
        write!(formatter, "variables:\n")?;
        for (index, variable) in self.variables.iter().enumerate() {
            write!(formatter, "{:?}: {:?}\n", VariableId(index), variable)?;
        }
        write!(formatter, "")
    }
}
