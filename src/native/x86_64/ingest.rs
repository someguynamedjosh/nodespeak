use num_enum::TryFromPrimitive;
use std::collections::HashMap;
use std::convert::TryFrom;

use super::structure as o;
use crate::specialized::structure as i;
use crate::trivial::structure::VariableType;

pub fn ingest(program: &i::Program) -> o::Program {
    let mut assembler = Assembler::new(program);
    assembler.entry_point()
}

#[derive(Clone, Copy, TryFromPrimitive)]
#[repr(usize)]
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

const NUM_REGISTERS: usize = 8;

impl Register {
    // What value to use to indicate this register in an opcode.
    fn opcode_index_u8(&self) -> u8 {
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
}

// Don't write variables to StorageAddress, it stores the start of our storage block.
const VARIABLE_REGISTERS: [Register; 7] = [
    Register::C,
    Register::B,
    Register::SP,
    Register::SI,
    Register::DI,
    // These have special meaning to some opcodes, they should be the last places we try to assign
    // variables to.
    Register::A,
    Register::D,
];

#[derive(Clone, Copy, Debug, PartialEq)]
enum RegisterContent {
    Empty,
    Temporary,
    Variable(i::VariableId),
}

struct Assembler<'a> {
    source: &'a i::Program,
    target: Vec<u8>,
    // Data type and address.
    inputs: Vec<(VariableType, usize)>,
    outputs: Vec<(VariableType, usize)>,
    // What variable is stored in each register.
    register_contents: [RegisterContent; NUM_REGISTERS],
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
            inputs: Vec::new(),
            outputs: Vec::new(),
            register_contents: [RegisterContent::Empty; NUM_REGISTERS],
            storage_size: 0,
            storage_locations: HashMap::new(),
        }
    }

    // Returns the address in the code block where the storage block's address is stored.
    fn write_prologue(&mut self) -> usize {
        // Push callee-saved registers to stack.
        self.write_bytes(&[0x53, 0x55, 0x41, 0x54, 0x41, 0x55, 0x41, 0x56, 0x41, 0x57]);
        // mov rbp imm64, the imm64 will eventually be overwritten with the storage block address.
        self.write_byte(0x48);
        self.write_byte(0xbd);
        let storage_address_index = self.target.len();
        self.write_64(0);
        self.storage_size += 8; // Allocate space to store rsp.
                                // mov [rbp] rsp so that we can restore the address of the stack after the function returns.
        self.write_bytes(&[0x48, 0x89, 0x65, 0x00]);
        // sub qword ptr [rpb] 0x08 because the following call instruction will push an 8-byte value
        // to the stack.
        self.write_bytes(&[0x48, 0x83, 0x6d, 0x00, 0x08]);
        // call disp32 to the actual body of the function.
        self.write_byte(0xe8);
        self.write_32(11); // Amount to jump.

        // Cleanup phase, 11 bytes.
        // rsp will already be restored because it needs to be restored before ret will work.
        // Pop callee-saved registers.
        self.write_bytes(&[0x41, 0x5f, 0x41, 0x5e, 0x41, 0x5d, 0x41, 0x5c, 0x5d, 0x5b]);
        // Return
        self.write_ret();

        storage_address_index
    }

    fn entry_point(&mut self) -> o::Program {
        let storage_address_index = self.write_prologue();

        for input in self.source.borrow_inputs().clone() {
            let vtype = self.source[input].get_type();
            let address = self.get_variable_address(input);
            self.inputs.push((vtype, address));
        }
        for output in self.source.borrow_outputs().clone() {
            let vtype = self.source[output].get_type();
            let address = self.get_variable_address(output);
            self.outputs.push((vtype, address));
        }
        for instruction in self.source.borrow_instructions().iter() {
            self.assemble_instruction(instruction)
        }
        // Make sure all outputs are stored to memory.
        for output in self.source.borrow_outputs() {
            let mut ops = vec![];
            for (register_index, contents) in self.register_contents.iter().enumerate() {
                if contents == &RegisterContent::Variable(*output) {
                    ops.push((Register::try_from(register_index).unwrap(), *output));
                }
            }
            for (register, var) in ops {
                self.write_mov_register_to_var32(register, var);
            }
        }
        // Return a value of 0 to indicate success.
        self.write_mov_imm32_to_register(Register::A, 0);
        self.write_restore_rsp_and_return();

        let mut program = o::Program::new(
            self.target.len(),
            self.storage_size,
            self.inputs.clone(),
            self.outputs.clone(),
        );
        // Write the address of the storage block into the code so that all the memory operations
        // load and store from the correct spot.
        // TODO: Check that we are on a 64 bit system.
        let storage_address = program.get_storage_address() as u64;
        let addr_bytes = storage_address.to_le_bytes();
        for (index, byte) in addr_bytes.iter().enumerate() {
            self.target[index + storage_address_index] = *byte;
        }
        unsafe {
            // This is safe because we have allocated enough bytes in the storage block to fit
            // the entire contents of self.target.
            program.write_iter_to_code(0, self.target.iter());
        }
        program
    }

    #[inline]
    fn write_byte(&mut self, byte: u8) {
        self.target.push(byte);
    }

    fn write_bytes(&mut self, bytes: &[u8]) {
        self.target.extend(bytes);
    }

    // arg1: first register, arg2: second register.
    fn write_modrm_two_register(&mut self, first: Register, second: Register) {
        // 0b11 indicates two registers.
        self.write_byte(0b11000000 | first.opcode_index_u8() << 3 | second.opcode_index_u8());
    }

    // arg1: register, arg2: [StorageAddress + disp32]
    fn write_modrm_reg_disp32(&mut self, register: Register) {
        // 0b10 with 0b101 as the last part indicates 32-bit displacement from EBP (StorageAddress)
        self.write_byte(0b10000101 | register.opcode_index_u8() << 3);
    }

    fn write_64(&mut self, value: u64) {
        // Everything in x86 goes from lowest order to highest order.
        self.write_byte(((value >> 0) % 0x100) as u8);
        self.write_byte(((value >> 8) % 0x100) as u8);
        self.write_byte(((value >> 16) % 0x100) as u8);
        self.write_byte(((value >> 24) % 0x100) as u8);
        self.write_byte(((value >> 32) % 0x100) as u8);
        self.write_byte(((value >> 40) % 0x100) as u8);
        self.write_byte(((value >> 48) % 0x100) as u8);
        self.write_byte(((value >> 56) % 0x100) as u8);
    }

    fn write_32(&mut self, value: u32) {
        // Everything in x86 goes from lowest order to highest order.
        self.write_byte(((value >> 0) % 0x100) as u8);
        self.write_byte(((value >> 8) % 0x100) as u8);
        self.write_byte(((value >> 16) % 0x100) as u8);
        self.write_byte(((value >> 24) % 0x100) as u8);
    }

    fn write_jmp_cond_disp8(&mut self, condition_code: u8, displacement: u8) {
        self.write_byte(0x70 | condition_code & 0x0F);
        self.write_byte(displacement);
    }

    fn write_mov_var32_to_register(&mut self, register: Register, variable: i::VariableId) {
        self.write_byte(0x8b);
        self.write_modrm_reg_disp32(register);
        let var_address = self.get_variable_address(variable) as u32;
        self.write_32(var_address);
    }

    fn write_mov_imm32_to_register(&mut self, register: Register, imm32: u32) {
        self.write_byte(0xb8 + register.opcode_index_u8());
        self.write_32(imm32);
    }

    fn write_mov_register_to_var32(&mut self, register: Register, variable: i::VariableId) {
        self.write_byte(0x89);
        self.write_modrm_reg_disp32(register);
        let var_address = self.get_variable_address(variable) as u32;
        self.write_32(var_address);
    }

    fn write_mov_imm8_to_register(&mut self, register: Register, imm8: u8) {
        self.write_byte(0xb0 + register.opcode_index_u8());
        self.write_byte(imm8);
    }

    fn write_mov_reg32_to_reg32(&mut self, from: Register, to: Register) {
        self.write_byte(0x89);
        self.write_modrm_two_register(from, to);
    }

    // Restores the stack pointer to the value saved before the start of the function. This must be
    // called before using the return instruction.
    fn write_restore_rsp(&mut self) {
        // mov rsp [rbp]
        self.write_bytes(&[0x48, 0x8b, 0x65, 0x00]);
    }

    fn write_ret(&mut self) {
        self.write_byte(0xc3);
    }

    fn write_restore_rsp_and_return(&mut self) {
        self.write_restore_rsp();
        self.write_ret();
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

    fn find_register_with(&self, contents: RegisterContent) -> Option<Register> {
        for register in &VARIABLE_REGISTERS {
            if self.register_contents[*register as usize] == contents {
                return Some(*register);
            }
        }
        None
    }

    fn find_empty_register(&mut self) -> Option<Register> {
        self.find_register_with(RegisterContent::Empty)
    }

    fn load_value_into_any(&mut self, value: &i::Value) -> Register {
        match value {
            i::Value::VariableAccess { variable, .. } => {
                if let Some(register) =
                    self.find_register_with(RegisterContent::Variable(*variable))
                {
                    register
                } else {
                    // TODO: Register spilling.
                    // TODO: Type checking stuff and arrays and all sorts of complicated tings.
                    let empty_register = self
                        .find_empty_register()
                        .expect("TODO: Register spilling.");
                    self.register_contents[empty_register as usize] =
                        RegisterContent::Variable(*variable);
                    self.write_mov_var32_to_register(empty_register, *variable);
                    empty_register
                }
            }
            i::Value::Literal(data) => {
                let binary_data = data.binary_data();
                let empty_register = self
                    .find_empty_register()
                    .expect("TODO: Register spilling.");
                self.register_contents[empty_register as usize] = RegisterContent::Temporary;
                self.write_mov_imm32_to_register(empty_register, binary_data);
                empty_register
            }
        }
    }

    fn load_value_into_register(&mut self, value: &i::Value, register: Register) -> Register {
        self.prepare_register_for_writing_safe(register);
        match value {
            i::Value::VariableAccess { variable, .. } => {
                if let Some(already_in) =
                    self.find_register_with(RegisterContent::Variable(*variable))
                {
                    self.write_mov_reg32_to_reg32(already_in, register);
                } else {
                    self.write_mov_var32_to_register(register, *variable);
                }
                self.register_contents[register as usize] = RegisterContent::Variable(*variable);
            }
            i::Value::Literal(data) => {
                let binary_data = data.binary_data();
                self.write_mov_imm32_to_register(register, binary_data);
                self.register_contents[register as usize] = RegisterContent::Temporary;
            }
        }
        register
    }

    fn load_zero_into_register(&mut self, register: Register) -> Register {
        self.prepare_register_for_writing_safe(register);
        // Xor the register with itself.
        self.write_byte(0x33);
        self.write_modrm_two_register(register, register);
        register
    }

    // Finds the registers where each sentenced variable lives and marks it as temporary. This
    // allows it to be a destination when using prepare_register_for_writing but ensures that no
    // values get copied into it before cleanup_temporaries is called.
    fn kill_variables(&mut self, kills: &[i::VariableId]) {
        for kill in kills {
            for index in 0..8 {
                if let RegisterContent::Variable(var_id) = self.register_contents[index] {
                    if &var_id == kill {
                        self.register_contents[index] = RegisterContent::Temporary;
                        break;
                    }
                }
            }
        }
    }

    // Marks that registers currently holding temporary values can be used again. This should
    // always be called after ensuring that no future operations will require the temporary values
    // contained in the registers.
    fn discard_temporary_registers(&mut self) {
        for register in &VARIABLE_REGISTERS {
            if self.register_contents[*register as usize] == RegisterContent::Temporary {
                self.register_contents[*register as usize] = RegisterContent::Empty;
            }
        }
    }

    // Ensures that the provided register can be written to without erasing data needed for later.
    // Note that if the provided register contains a temporary, writing to it will destroy it. If
    // you still need a temporary after the write operation, you must perform a check to make sure
    // you aren't overwriting it yourself.
    fn prepare_register_for_writing(&mut self, register: Register) {
        // TODO: Register spilling.
        match self.register_contents[register as usize] {
            RegisterContent::Variable(id) => {
                let empty_register = self
                    .find_empty_register()
                    .expect("TODO: Register spilling.");
                self.write_mov_reg32_to_reg32(register, empty_register);
                self.register_contents[empty_register as usize] = RegisterContent::Variable(id);
            }
            RegisterContent::Temporary => {
                self.register_contents[register as usize] = RegisterContent::Empty
            }
            RegisterContent::Empty => (),
        }
    }

    // Like above, but causes an error if attempting to write to a register containing a temporary.
    // Use this it you cannot guarantee that temporary values will not be needed later.
    fn prepare_register_for_writing_safe(&mut self, register: Register) {
        match self.register_contents[register as usize] {
            RegisterContent::Temporary => {
                panic!("Cannot guarantee that this temporary value will not be needed later.")
            }
            _ => self.prepare_register_for_writing(register),
        }
    }

    // If value is a simple variable, just labels the register as containing that variable.
    // (TODO: If value is an array access, then write to that element.)
    fn commit_value_in_register(&mut self, value: &i::Value, register: Register) {
        // TODO: Complicated array stuff.
        if let i::Value::VariableAccess { variable, .. } = value {
            let new_content = RegisterContent::Variable(*variable);
            // If any other register claims to contain this value, it's invalid now.
            for register in &VARIABLE_REGISTERS {
                if self.register_contents[*register as usize] == new_content {
                    self.register_contents[*register as usize] = RegisterContent::Empty;
                }
            }
            self.register_contents[register as usize] = RegisterContent::Variable(*variable);
        }
    }

    fn assemble_instruction(&mut self, instruction: &i::AnnotatedInstruction) {
        // Order of operations:
        // load_values    - ensures that the values required for the operation are loaded into
        //                  registers.
        // kill_variables - if a variable will die after this instruction, we don't need to worry
        //                  about keeping it around. Marks registers containing variables sentenced
        //                  to death as Temporary.
        // prep_reg_for_w - call this for all registers that will be modified. If it
        //                  contains a non-temporary value, ensures that that value will live past
        //                  the operation.
        // Call whatever functions necessary to create the byte code for the operation.
        // disc_temp_regs - changes registers marked as temporary to be marked as empty.
        // cmt_val_in_reg - call this for every register written to. Indicates what value the
        //                  register represents.
        match &instruction.instruction {
            i::Instruction::Move { from, to } => {
                // TODO: This is only simple right now because we aren't handling arrays.
                let reg_from = self.load_value_into_any(from);
                // We aren't actually writing to it, just giving it a new label. But then it might
                // be changed, so if from didn't get killed, we need to make sure it is saved for
                // later.
                self.kill_variables(&instruction.kills);
                self.prepare_register_for_writing(reg_from);
                self.commit_value_in_register(to, reg_from);
                self.discard_temporary_registers();
            }
            i::Instruction::BinaryOperation { op, a, b, x } => {
                // TODO: Optimize.
                self.kill_variables(&instruction.kills);
                match op {
                    i::BinaryOperator::AddI => {
                        let reg_a = self.load_value_into_any(a);
                        let reg_b = self.load_value_into_any(b);
                        self.prepare_register_for_writing(reg_a);
                        self.write_byte(0x01); // Add to second operand.
                        self.write_modrm_two_register(reg_b, reg_a);
                        self.discard_temporary_registers();
                        self.commit_value_in_register(x, reg_a);
                    }
                    i::BinaryOperator::SubI => {
                        let reg_a = self.load_value_into_any(a);
                        let reg_b = self.load_value_into_any(b);
                        self.prepare_register_for_writing(reg_a);
                        self.write_byte(0x29); // Subtract from second operand.
                        self.write_modrm_two_register(reg_b, reg_a);
                        self.discard_temporary_registers();
                        self.commit_value_in_register(x, reg_a);
                    }
                    i::BinaryOperator::MulI => {
                        let reg_a = self.load_value_into_any(a);
                        let reg_b = self.load_value_into_any(b);
                        self.prepare_register_for_writing(reg_a);
                        self.write_bytes(&[0x0f, 0xaf]); // Multiply into first operand.
                        self.write_modrm_two_register(reg_a, reg_b);
                        self.discard_temporary_registers();
                        self.commit_value_in_register(x, reg_a);
                    }
                    i::BinaryOperator::DivI => {
                        // The divide instruction requires the first operand to be contained in the
                        // a and d registers, with d containing the high bits and a containing the
                        // low bits.
                        let reg_a = self.load_value_into_register(a, Register::A);
                        self.load_zero_into_register(Register::D);
                        let reg_b = self.load_value_into_any(b);
                        self.prepare_register_for_writing(reg_a);
                        // Divide by second operand, first operand is ignored.
                        self.write_bytes(&[0xF7]);
                        self.write_byte(0b11111000 | reg_b.opcode_index_u8());
                        self.discard_temporary_registers();
                        self.commit_value_in_register(x, reg_a);
                    }
                    i::BinaryOperator::ModI => {
                        // Same logic as above, because the divide instruction stores the remainder
                        // in the D register.
                        self.load_value_into_register(a, Register::A);
                        let reg_remainder = self.load_zero_into_register(Register::D);
                        let reg_b = self.load_value_into_any(b);
                        self.prepare_register_for_writing(reg_remainder);
                        // Divide by second operand, first operand is ignored.
                        self.write_bytes(&[0xF7]);
                        self.write_byte(0b11111000 | reg_b.opcode_index_u8());
                        self.discard_temporary_registers();
                        self.commit_value_in_register(x, reg_remainder);
                    }
                    _ => unimplemented!("{:?}", op),
                }
            }
            i::Instruction::Compare { a, b } => {
                let reg_a = self.load_value_into_any(a);
                let reg_b = self.load_value_into_any(b);
                self.kill_variables(&instruction.kills);
                self.write_byte(0x39); // Compare two registers.
                self.write_modrm_two_register(reg_a, reg_b);
                self.discard_temporary_registers();
            }
            i::Instruction::Assert(condition) => {
                self.kill_variables(&instruction.kills);
                // Skip over the following pieces of code if the condition was true.
                self.write_jmp_cond_disp8(condition.code(), 10);
                // Return a value of 1 to indicate failure.
                self.write_mov_imm32_to_register(Register::A, 1); // 5 bytes
                self.write_restore_rsp_and_return(); // 5 bytes
                self.discard_temporary_registers();
            }
        }
    }
}
