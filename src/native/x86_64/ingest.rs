use std::collections::HashMap;

use super::structure as o;
use crate::specialized::structure as i;

pub fn ingest(program: &i::Program) -> o::Program {
    let mut assembler = Assembler::new(program);
    assembler.entry_point()
}

#[derive(Clone, Copy)]
enum Register {
    A,
    C,
    D,
    B,
    SP,
    StorageAddress,
    SI,
    DI,
}

impl Register {
    fn index(&self) -> u8 {
        match self {
            Self::A => 0,
            Self::C => 1,
            Self::D => 2,
            Self::B => 3,
            Self::SP => 4,
            Self::StorageAddress => 5,
            Self::SI => 6,
            Self::DI => 7,
        }
    }

    fn from_index(index: u8) -> Self {
        match index {
            0 => Self::A,
            1 => Self::C,
            2 => Self::D,
            3 => Self::B,
            4 => Self::SP,
            5 => Self::StorageAddress,
            6 => Self::SI,
            7 => Self::DI,
            _ => unreachable!("{} is not a valid index for a register.", index),
        }
    }
}

struct Assembler<'a> {
    source: &'a i::Program,
    target: Vec<u8>,
    // What variable is stored in each register.
    register_contents: [Option<i::VariableId>; 8],
    // Stores where addresses of different variables should be stored.
    displacement_pointers: Vec<(usize, i::VariableId)>,
    // How much space should be allocated for variable storage.
    storage_size: usize,
    // The position each variable occupies in storage.
    storage_locations: HashMap<i::VariableId, usize>,
}

impl<'a> Assembler<'a> {
    fn new(source: &i::Program) -> Assembler {
        Assembler {
            source,
            target: Vec::new(),
            register_contents: [None; 8],
            displacement_pointers: Vec::new(),
            storage_size: 0,
            storage_locations: HashMap::new(),
        }
    }

    fn entry_point(&mut self) -> o::Program {
        for instruction in self.source.borrow_instructions().iter() {
            self.assemble_instruction(instruction)
        }
        self.ret();
        self.finalize()
    }

    fn finalize(&mut self) -> o::Program {
        let mut program = o::Program::new(self.target.len(), self.storage_size);
        program.write_iter_to_code(0, self.target.iter());
        println!("{:x?}", program.get_storage_address(0));
        program
    }

    #[inline]
    fn write_byte(&mut self, byte: u8) {
        self.target.push(byte);
    }

    // arg1: first register, arg2: second register.
    fn write_modrm_two_register(&mut self, first: Register, second: Register) {
        // 0b11 indicates two registers.
        self.write_byte(0b11000000 | first.index() << 3 | second.index());
    }

    // arg1: register, arg2: [StorageAddress + disp32]
    fn write_modrm_reg_disp32(&mut self, register: Register) {
        // 0b10 with 0b101 as the last part indicates 32-bit displacement from EBP (StorageAddress)
        self.write_byte(0b10000101 | register.index() << 3);
    }

    fn write_disp32(&mut self, value: u32) {
        // Everything in x86 goes from lowest order to highest order.
        self.write_byte(((value >> 0) % 0x100) as u8);
        self.write_byte(((value >> 8) % 0x100) as u8);
        self.write_byte(((value >> 16) % 0x100) as u8);
        self.write_byte(((value >> 24) % 0x100) as u8);
    }

    fn write_imm32(&mut self, value: u32) {
        self.write_byte(((value >> 0) % 0x100) as u8);
        self.write_byte(((value >> 8) % 0x100) as u8);
        self.write_byte(((value >> 16) % 0x100) as u8);
        self.write_byte(((value >> 24) % 0x100) as u8);
    }

    fn mov_var32_to_register(&mut self, register: Register, variable: i::VariableId) {
        self.write_byte(0x8b);
        self.write_modrm_reg_disp32(register);
        let var_address = self.get_variable_address(variable) as u32;
        self.write_disp32(var_address);
    }

    fn mov_imm32_to_register(&mut self, register: Register, imm32: u32) {
        self.write_byte(0xb8 + register.index());
        self.write_imm32(imm32);
    }

    fn mov_imm8_to_register(&mut self, register: Register, imm8: u8) {
        self.write_byte(0xb0 + register.index());
        self.write_byte(imm8);
    }

    fn ret(&mut self) {
        self.write_byte(0xc3);
    }

    // Returns the address of the specified variable relative to the start of
    // the storage block.
    fn get_variable_address(&mut self, variable: i::VariableId) -> usize {
        if let Some(address) = self.storage_locations.get(&variable) {
            *address
        } else {
            let address = self.storage_size;
            // TODO: Align address.
            // TODO: Proper sizes for different variable types.
            self.storage_size += 4;
            self.storage_locations.insert(variable, address);
            address
        }
    }

    fn load_values(&mut self, values: &[&i::Value]) -> Vec<Register> {
        let mut assigned_registers = vec![None; values.len()];
        let mut registers_used = [false; 8];
        let mut unloaded_variables = Vec::new();
        let find_empty_register =
            |registers_used: &[bool], register_contents: &[Option<i::VariableId>]| {
                registers_used
                    .iter()
                    .zip(register_contents.iter())
                    .position(|(used, contents)| !used && contents.is_none())
            };
        // TODO: Complicated indexing stuff.
        // TODO: Register spilling.
        // Check for any values that are already loaded into registers.
        for (value_index, value) in values.iter().enumerate() {
            if let i::Value::VariableAccess { variable, .. } = value {
                let mut success = false;
                for (register_index, register_contents) in self.register_contents.iter().enumerate()
                {
                    if register_contents == &Some(*variable) {
                        registers_used[register_index] = true;
                        assigned_registers[value_index] =
                            Some(Register::from_index(register_index as u8));
                        success = true;
                        break;
                    }
                }
                if !success {
                    unloaded_variables.push((value_index, *variable));
                }
            }
        }
        // Copy in any variables not currently in the registers.
        for (value_index, variable) in unloaded_variables {
            // TODO: Register spilling.
            let empty_index = find_empty_register(&registers_used, &self.register_contents)
                .expect("No empty registers available.");
            registers_used[empty_index] = true;
            let empty_register = Register::from_index(empty_index as u8);
            assigned_registers[value_index] = Some(empty_register);
            self.register_contents[empty_index] = Some(variable);
            self.mov_var32_to_register(empty_register, variable);
        }
        // Copy literals into registers.
        for (value_index, value) in values.iter().enumerate() {
            if let i::Value::Literal(data) = value {
                let binary_data = match data {
                    i::LiteralData::Bool(data) => {
                        if *data {
                            1
                        } else {
                            0
                        }
                    }
                    // TODO: Check for signed values.
                    i::LiteralData::Int(data) => *data as i32 as u32,
                    i::LiteralData::Float(data) => f32::to_bits(*data as f32),
                };
                let empty_index = find_empty_register(&registers_used, &self.register_contents)
                    .expect("No empty registers available.");
                registers_used[empty_index] = true;
                let empty_register = Register::from_index(empty_index as u8);
                assigned_registers[value_index] = Some(empty_register);
                self.mov_imm32_to_register(empty_register, binary_data);
            }
        }
        assigned_registers
            .into_iter()
            .map(|value| value.unwrap())
            .collect()
    }

    fn kill_variables(&mut self, kills: &[i::VariableId]) {
        return;
        for kill in kills {
            for index in 0..8 {
                if let Some(var_id) = self.register_contents[index] {
                    if &var_id == kill {
                        self.register_contents[index] = None;
                        break;
                    }
                }
            }
        }
    }

    // Makes sure the provided register can be written to without messing up some later part of the
    // program.
    fn clear_register(&mut self, register: Register) {
        // TODO: Register spilling.
        self.register_contents[register.index() as usize] = None;
    }

    // Marks the provided register as containing the provided value.
    fn store_value(&mut self, value: &i::Value, register: Register) {
        // TODO: Complicated array stuff.
        if let i::Value::VariableAccess { variable, .. } = value {
            self.register_contents[register.index() as usize] = Some(*variable);
        }
    }

    fn store_register_in_value(&mut self, register: Register, value: &i::Value) {
        // TODO: Complicated array stuff.
        if let i::Value::VariableAccess { variable, .. } = value {
            for register_index in 0..8 {
                if self.register_contents[register_index].is_none() {
                    self.write_byte(0x89); // Transfer reg1 to reg2.
                    self.write_modrm_two_register(
                        register,
                        Register::from_index(register_index as u8),
                    );
                    self.register_contents[register_index] = Some(*variable);
                }
            }
        }
    }

    fn assemble_instruction(&mut self, instruction: &i::AnnotatedInstruction) {
        match &instruction.instruction {
            i::Instruction::Move { from, to } => {
                // TODO: This is only simple right now because we aren't handling arrays.
                let registers = self.load_values(&[from]);
                self.kill_variables(&instruction.kills);
                // TODO: Complicated indexing stuff.
                let mut source_is_killed = false;
                if let i::Value::VariableAccess { variable, .. } = from {
                    source_is_killed = instruction.kills.iter().any(|value| value == variable);
                }
                // If source is killed, we can just say the source register contains the target
                // value.
                if source_is_killed {
                    self.store_value(to, registers[0]);
                } else {
                    self.store_register_in_value(registers[0], to);
                }
            }
            i::Instruction::BinaryOperation { op, a, b, x } => {
                // TODO: Optimize.
                let mut registers = self.load_values(&[a, b]);
                if b == x {
                    let temp = registers[0];
                    registers[0] = registers[1];
                    registers[1] = temp;
                }
                self.kill_variables(&instruction.kills);
                match op {
                    i::BinaryOperator::AddI => {
                        self.write_byte(0x01);
                        self.write_modrm_two_register(registers[0], registers[1]);
                    }
                    _ => unimplemented!("{:?}", op),
                }
                // reg1 should be overwritten.
                self.store_value(x, registers[1]);
            }
            _ => unimplemented!(),
        }
        println!("   {:?}", self.register_contents);
    }
}
