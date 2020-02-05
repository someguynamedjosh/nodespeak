use super::{Value, Variable};
use std::fmt::{self, Debug, Formatter};
use std::ops::{Index, IndexMut};

#[derive(Clone, Copy, Eq, Hash, PartialEq)]
pub struct VariableId(usize);

impl Debug for VariableId {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        write!(formatter, "v{}", self.0)
    }
}

pub trait Instruction {
    fn reads_from(&self) -> Vec<&Value>;
    fn writes_to(&self) -> Vec<&Value>;
}

pub struct WrappedInstruction<IType> {
    instruction: IType,
    kills: Vec<VariableId>, // Which variables are no longer needed after this instruction.
    births: Vec<VariableId>, // Which variables come alive after this instruction.
                            // Births happen after kills.
}

enum VariableStatus {
    Unused,
    Read(usize),
    Written(usize),
}

pub struct GenericProgram<IType: Instruction> {
    variables: Vec<Variable>,
    var_statuses: Vec<VariableStatus>,
    instructions: Vec<WrappedInstruction<IType>>,
}

impl<IType: Instruction> Index<VariableId> for GenericProgram<IType> {
    type Output = Variable;

    fn index(&self, variable: VariableId) -> &Self::Output {
        &self.variables[variable.0]
    }
}

impl<IType: Instruction> IndexMut<VariableId> for GenericProgram<IType> {
    fn index_mut(&mut self, variable: VariableId) -> &mut Self::Output {
        &mut self.variables[variable.0]
    }
}

impl<IType: Instruction> GenericProgram<IType> {
    pub fn new() -> GenericProgram<IType> {
        GenericProgram {
            variables: Vec::new(),
            var_statuses: Vec::new(),
            instructions: Vec::new(),
        }
    }

    pub fn adopt_variable(&mut self, variable: Variable) -> VariableId {
        let id = VariableId(self.variables.len());
        self.variables.push(variable);
        self.var_statuses.push(VariableStatus::Unused);
        id
    }

    pub fn add_instruction(&mut self, instruction: IType) {
        let mut reads = vec![];
        let mut writes = vec![];
        // TODO: Really complicated things to make indexed variables work.
        for read in instruction.reads_from() {
            if let Value::VariableAccess { variable, .. } = read {
                reads.push(*variable);
            }
        }
        for write in instruction.writes_to() {
            if let Value::VariableAccess { variable, .. } = write {
                writes.push(*variable);
            }
        }

        let new_index = self.instructions.len();
        self.instructions.push(WrappedInstruction {
            instruction,
            kills: vec![],
            births: vec![],
        });

        // This basically does liveness analysis on variables. Every time a variable is written to
        // (and it will get used later), the corresponding instruction will have that variable in
        // its births list. Every time a variable is read for the last time before its value is not
        // needed any more, the instruction where it is read will have that variable added to its
        // kills list.
        for var in reads {
            if let VariableStatus::Written(old_index) = self.var_statuses[var.0] {
                self.instructions[old_index].births.push(var);
            }
            self.var_statuses[var.0] = VariableStatus::Read(new_index);
        }
        for var in writes {
            if let VariableStatus::Read(old_index) = self.var_statuses[var.0] {
                self.instructions[old_index].kills.push(var);
            }
            self.var_statuses[var.0] = VariableStatus::Written(new_index);
        }
    }

    // Call this after all instructions have been added.
    // TODO: Possibly create a program builder so that this logic can be seperated from the actual
    // program?
    pub fn complete_liveness_analysis(&mut self) {
        for (index, status) in self.var_statuses.iter().enumerate() {
            if let VariableStatus::Read(old_index) = status {
                self.instructions[*old_index].kills.push(VariableId(index));
            }
        }
    }

    pub fn borrow_instructions(&self) -> &Vec<WrappedInstruction<IType>> {
        &self.instructions
    }
}

impl<IType: Instruction + Debug> Debug for GenericProgram<IType> {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        write!(formatter, "instructions:\n")?;
        for instruction in &self.instructions {
            write!(formatter, "{:?}\n", instruction.instruction)?;
            if instruction.kills.len() > 0 {
                write!(formatter, "  kills")?;
                for var in instruction.kills.iter() {
                    write!(formatter, " {:?}", var)?;
                }
                write!(formatter, "\n")?;
            }
            if instruction.births.len() > 0 {
                write!(formatter, "  births")?;
                for var in instruction.births.iter() {
                    write!(formatter, " {:?}", var)?;
                }
                write!(formatter, "\n")?;
            }
        }
        write!(formatter, "variables:\n")?;
        for (index, variable) in self.variables.iter().enumerate() {
            write!(formatter, "{:?}: {:?}\n", VariableId(index), variable)?;
        }
        write!(formatter, "")
    }
}
