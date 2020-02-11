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

#[derive(Clone, Copy, PartialEq)]
enum RegisterNamespace {
    Primary,
    XMM,
}

impl RegisterNamespace {
    fn is_compatible_with(&self, data_type: VariableType) -> bool {
        match data_type {
            VariableType::I32 => *self == Self::Primary,
            VariableType::F32 => *self == Self::XMM,
            VariableType::B8 => *self == Self::Primary,
        }
    }
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
    XMM0,
    XMM1,
    XMM2,
    XMM3,
    XMM4,
    XMM5,
    XMM6,
    XMM7,
}

const NUM_REGISTERS: usize = 16;

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
            Self::XMM0 => 0,
            Self::XMM1 => 1,
            Self::XMM2 => 2,
            Self::XMM3 => 3,
            Self::XMM4 => 4,
            Self::XMM5 => 5,
            Self::XMM6 => 6,
            Self::XMM7 => 7,
        }
    }

    // Returns the 'namespace' of the register. The idea is that registers in different namespaces
    // use different sets of instructions to manipulate them.
    fn get_namespace(&self) -> RegisterNamespace {
        match self {
            Self::A
            | Self::C
            | Self::D
            | Self::B
            | Self::SP
            | Self::StorageAddress
            | Self::SI
            | Self::DI => RegisterNamespace::Primary,
            Self::XMM0
            | Self::XMM1
            | Self::XMM2
            | Self::XMM3
            | Self::XMM4
            | Self::XMM5
            | Self::XMM6
            | Self::XMM7 => RegisterNamespace::XMM,
        }
    }

    fn is_primary(&self) -> bool {
        self.get_namespace() == RegisterNamespace::Primary
    }

    fn is_xmm(&self) -> bool {
        self.get_namespace() == RegisterNamespace::XMM
    }

    fn is_compatible_with(&self, data_type: VariableType) -> bool {
        self.get_namespace().is_compatible_with(data_type)
    }
}

// Don't write variables to StorageAddress, it stores the start of our storage block. Also, A and
// D have special meanings for some instructions, so they should be given low priority.
const INT_REGISTERS: [Register; 7] = [
    Register::C,
    Register::B,
    Register::SP,
    Register::SI,
    Register::DI,
    Register::A,
    Register::D,
];

const FLOAT_REGISTERS: [Register; 8] = [
    Register::XMM0,
    Register::XMM1,
    Register::XMM2,
    Register::XMM3,
    Register::XMM4,
    Register::XMM5,
    Register::XMM6,
    Register::XMM7,
];

// A list of all registers that can potentially contain the value of a variable.
const ALL_VARIABLE_REGISTERS: [Register; 15] = [
    Register::C,
    Register::B,
    Register::SP,
    Register::SI,
    Register::DI,
    Register::A,
    Register::D,
    Register::XMM0,
    Register::XMM1,
    Register::XMM2,
    Register::XMM3,
    Register::XMM4,
    Register::XMM5,
    Register::XMM6,
    Register::XMM7,
];

#[derive(Clone, Copy, Debug, PartialEq)]
enum RegisterContent {
    /// Indicates the register contains nothing of importance.
    Empty,
    /// Indicates the register contains a value that will no longer be needed once the register is
    /// unlocked.
    Temporary,
    /// Indicates the register contains the specified variable and that it will still be needed some
    /// time in the future after the retister is unlocked.
    Variable(i::VariableId),
}

struct Assembler<'a> {
    source: &'a i::Program,
    target: Vec<u8>,
    // Data type and address.
    inputs: Vec<(VariableType, usize)>,
    outputs: Vec<(VariableType, usize)>,
    /// What variable is stored in each register.
    register_contents: [RegisterContent; NUM_REGISTERS],
    /// Whether each register is locked. If a register is locked, its contents should not be
    /// spilled / overwritten until it is unlocked.
    register_locked: [bool; NUM_REGISTERS],
    /// How much space should be allocated for variable storage.
    storage_size: usize,
    /// The position each variable occupies in storage.
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
            register_locked: [false; NUM_REGISTERS],
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

    // Functions beginning with write_ add machine code to the output. Functions beginning with
    // meta_ store or retrieve some piece of metadata about what the current state of the program
    // is. These functions add no overhead to the actual end result, they just act as helpers to
    // ensure that the program will work as intended once assembled. Functions that begin with
    // neither of these things perform a mix of these two functions as explained on a per-function
    // basis, or alternatively serve some entirely unrelated purpose.
    // Different things meta / hybrid functions can do to registers:
    // Lock:    a locked register will not be spilled or overwritten by other helper functions.
    // Unlock:  unlocks a register. If the register contained a temporary value, then unlocking
    //          labels the register as being empty.
    // Prepare: ensures that a register can be written to without stomping on data that will
    //          be used later in the program.
    // Write:   Prepares and locks a register while adding assembly code to transfer a value
    //          into it.

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
        debug_assert!(register.is_compatible_with(self.source[variable].get_type()));
        match register.get_namespace() {
            RegisterNamespace::Primary => self.write_byte(0x8b),
            // Load value from memory into xmm register.
            RegisterNamespace::XMM => self.write_bytes(&[0xf3, 0x0f, 0x10]),
        }
        self.write_modrm_reg_disp32(register);
        let var_address = self.get_variable_address(variable) as u32;
        self.write_32(var_address);
    }

    fn write_mov_imm32_to_register(&mut self, register: Register, imm32: u32) {
        match register.get_namespace() {
            RegisterNamespace::Primary => {
                self.write_byte(0xb8 + register.opcode_index_u8());
                self.write_32(imm32);
            }
            RegisterNamespace::XMM => {
                // There is no way to load immediates to XMM. Instead, add a constant in to the
                // storage block and load that.
                // TODO: Allow writing constants to storage.
                unimplemented!();
                // Write memory contents to xmm register.
                self.write_bytes(&[0xf3, 0x0f, 0x10]);
            }
        }
    }

    fn write_mov_register_to_var32(&mut self, register: Register, variable: i::VariableId) {
        debug_assert!(register.is_compatible_with(self.source[variable].get_type()));
        match register.get_namespace() {
            RegisterNamespace::Primary => self.write_byte(0x89),
            // Write first operand (xmm reg) to second operand (memory)
            RegisterNamespace::XMM => self.write_bytes(&[0xf3, 0x0f, 0x11]),
        }
        self.write_modrm_reg_disp32(register);
        let var_address = self.get_variable_address(variable) as u32;
        self.write_32(var_address);
    }

    fn write_mov_reg32_to_reg32(&mut self, from: Register, to: Register) {
        debug_assert!(from.get_namespace() == to.get_namespace());
        match from.get_namespace() {
            RegisterNamespace::Primary => self.write_byte(0x89),
            // Write first operand (xmm reg) to second operand (xmm reg)
            RegisterNamespace::XMM => self.write_bytes(&[0xf3, 0x0f, 0x11]),
        }
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

    /// Returns the address of the specified variable relative to the start of
    /// the storage block.
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

    fn meta_find_register_from_list_containing(
        &self,
        list: &[Register],
        contents: RegisterContent,
    ) -> Option<Register> {
        for register in list {
            if self.register_contents[*register as usize] == contents {
                return Some(*register);
            }
        }
        None
    }

    fn meta_find_register_containing(&self, contents: RegisterContent) -> Option<Register> {
        self.meta_find_register_from_list_containing(&ALL_VARIABLE_REGISTERS, contents)
    }

    /// Returns Some(register) if a register is found in the list which is not locked and has
    /// empty contents, or None if none could be found.
    fn meta_find_unused_register_from_list(&self, list: &[Register]) -> Option<Register> {
        for register in list {
            if self.register_contents[*register as usize] == RegisterContent::Empty
                && !self.register_locked[*register as usize]
            {
                return Some(*register);
            }
        }
        None
    }

    /// Returns Some(register) if an integer register is found which is not locked and has
    /// empty contents, or None if none could be found.
    fn meta_find_unused_int_register(&mut self) -> Option<Register> {
        self.meta_find_unused_register_from_list(&INT_REGISTERS)
    }


    /// Returns Some(register) if a float register is found which is not locked and has
    /// empty contents, or None if none could be found.
    fn meta_find_unused_float_register(&mut self) -> Option<Register> {
        self.meta_find_unused_register_from_list(&FLOAT_REGISTERS)
    }

    /// Marks the specified register as locked. This will cause other helper functions to avoid
    /// overwriting or spilling the register until it is unlocked. Safe to call on an already
    /// locked register.
    fn meta_lock_register(&mut self, register: Register) {
        self.register_locked[register as usize] = true;
    }

    /// Marks the specified register as unlocked. If the register contained a temporary value,
    /// marks the register as being empty. Should only be called on locked registers (protected by
    /// debug assert.)
    fn meta_unlock_register(&mut self, register: Register) {
        debug_assert!(self.register_locked[register as usize]);
        self.register_locked[register as usize] = false;
        if self.register_contents[register as usize] == RegisterContent::Temporary {
            self.register_contents[register as usize] = RegisterContent::Empty;
        }
    }

    /// Unlocks all currently locked registers. (See meta_unlock_register.)
    fn meta_unlock_registers(&mut self) {
        for register in &ALL_VARIABLE_REGISTERS {
            if self.register_locked[*register as usize] {
                self.meta_unlock_register(*register);
            }
        }
    }

    /// Finds the registers where each sentenced variable lives and marks it as temporary. This
    /// ensures that the register can be re-used once the register is unlocked.
    fn meta_kill_variables(&mut self, kills: &[i::VariableId]) {
        for kill in kills {
            for register in &ALL_VARIABLE_REGISTERS {
                let index = *register as usize;
                if let RegisterContent::Variable(var_id) = self.register_contents[index] {
                    if &var_id == kill {
                        self.register_contents[index] = RegisterContent::Temporary;
                        break;
                    }
                }
            }
        }
    }

    /// Ensures that the provided register can be written to without erasing data needed for later.
    /// This should only be used before a single instruction will read from and then write to a 
    /// locked register.
    fn prepare_locked_register(&mut self, register: Register) {
        match self.register_contents[register as usize] {
            RegisterContent::Variable(id) => {
                let data_type = self.source[id].get_type();
                let write_into = self.prepare_any_register_for(data_type);
                self.write_mov_reg32_to_reg32(register, write_into);
                self.register_contents[write_into as usize] = RegisterContent::Variable(id);
                self.register_contents[register as usize] = RegisterContent::Empty
            }
            RegisterContent::Temporary => {
                self.register_contents[register as usize] = RegisterContent::Empty
            }
            RegisterContent::Empty => (),
        }
    }

    /// Ensures that the provided register can be written to without erasing data needed for later.
    /// Similar to prepare_locked_register, but will panic if called on a locked register.
    fn prepare_register(&mut self, register: Register) {
        debug_assert!(
            !self.register_locked[register as usize],
            "Cannot write to a locked register."
        );
        self.prepare_locked_register(register);
    }

    /// Finds a register capable of storing the provided data type, and ensures that it can be
    /// written to without overwriting any data that the program needs for later. The register is 
    /// marked as being empty.
    /// Meta: searches for any unused registers that can hold the specified data type.
    /// Write: potentially writes instructions to spill a register if no unused register is found.
    fn prepare_any_register_for(&mut self, data_type: VariableType) -> Register {
        // TODO: Register spilling.
        if data_type == VariableType::I32 {
            let unused_register = self.meta_find_unused_int_register();
            if let Some(register) = unused_register {
                register
            } else {
                unimplemented!("TODO: Register spilling.")
            }
        } else if data_type == VariableType::F32 {
            let unused_register = self.meta_find_unused_float_register();
            if let Some(register) = unused_register {
                register
            } else {
                unimplemented!("TODO: Register spilling.")
            }
        } else {
            unimplemented!()
        }
    }

    /// Loads the specified value into a compatible register.
    /// Write: instructions to load the provided value into a compatible register. Potentially
    /// adds code to spill a register so that the value can be loaded.
    /// Meta: marks the provided register as containing the provided value. If the value was a
    /// literal, the register is marked as containing a temporary value. The register is also locked
    /// in either case such that its contents cannot be spilled until it is unlocked again.
    fn load_value_into_any(&mut self, value: &i::Value) -> Register {
        match value {
            i::Value::VariableAccess { variable, .. } => {
                if let Some(register) =
                    self.meta_find_register_containing(RegisterContent::Variable(*variable))
                {
                    self.meta_lock_register(register);
                    register
                } else {
                    // TODO: Register spilling.
                    // TODO: arrays and all sorts of complicated tings.
                    let data_type = self.source[*variable].get_type();
                    let write_into = self.prepare_any_register_for(data_type);
                    self.write_mov_var32_to_register(write_into, *variable);
                    self.register_contents[write_into as usize] =
                        RegisterContent::Variable(*variable);
                    self.meta_lock_register(write_into);
                    write_into
                }
            }
            i::Value::Literal(data) => {
                let data_type = data.get_type();
                let binary_data = data.binary_data();
                let write_into = self.prepare_any_register_for(data_type);
                self.write_mov_imm32_to_register(write_into, binary_data);
                self.register_contents[write_into as usize] = RegisterContent::Temporary;
                self.meta_lock_register(write_into);
                write_into
            }
        }
    }

    /// Loads the specified value into the specified register. This should not be called on a locked
    /// register (protected by debug assert.)
    /// Write: instructions to load the provided value into the provided register. Potentially
    /// adds code to spill the register so that the value can be loaded.
    /// Meta: marks the provided register as containing the provided value. If the value was a
    /// literal, the register is marked as containing a temporary value. The register is also marked
    /// as locked in either case such that its contents cannot be spilled until it's unlocked again.
    fn load_value_into_register(&mut self, value: &i::Value, register: Register) -> Register {
        self.prepare_register(register);
        match value {
            i::Value::VariableAccess { variable, .. } => {
                if let Some(already_in) =
                    self.meta_find_register_containing(RegisterContent::Variable(*variable))
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

    /// Sets the value of the register to zero. This should not be called on a locked register
    /// (protected by debug assert.)
    /// Meta: prepares and locks the specified register. Marks the register as containing a
    /// temporary value.
    /// Write: code to xor a register with itself.
    fn load_zero_into_register(&mut self, register: Register) -> Register {
        // TODO: zero xmm register.
        debug_assert!(register.get_namespace() == RegisterNamespace::Primary);
        self.prepare_register(register);
        // Xor the register with itself.
        self.write_byte(0x33);
        self.write_modrm_two_register(register, register);
        self.register_contents[register as usize] = RegisterContent::Temporary;
        self.meta_lock_register(register);
        register
    }

    /// If value is a simple variable, just labels the register as containing that variable.
    /// (TODO: If value is an array access, then write to that element.)
    fn commit_value_in_register(&mut self, value: &i::Value, register: Register) {
        // TODO: Complicated array stuff.
        if let i::Value::VariableAccess { variable, .. } = value {
            debug_assert!(register.is_compatible_with(self.source[*variable].get_type()));
            let new_content = RegisterContent::Variable(*variable);
            // If any other register claims to contain this value, it's invalid now.
            for register in &ALL_VARIABLE_REGISTERS {
                if self.register_contents[*register as usize] == new_content {
                    self.register_contents[*register as usize] = RegisterContent::Empty;
                }
            }
            self.register_contents[register as usize] = RegisterContent::Variable(*variable);
        }
    }

    fn assemble_instruction(&mut self, instruction: &i::AnnotatedInstruction) {
        // General template:
        // load_* functions such as load_value_into_any.
        // meta_kill_variables to mark any sentenced variables as temporary to this instruction.
        // prepare_locked_register for any registers that will be overwritten by the instruction.
        // write_* to write the machine code to execute the instruction.
        // meta_unlock_registers so that they can be used/overwritten/spilled by future instructions
        //     and so that registers containing temporary values can be reused.
        // commit_value_in_register to ensure that we are properly keeping track of the contents
        //     of any modified registers.
        match &instruction.instruction {
            i::Instruction::Move { from, to } => {
                // TODO: This is only simple right now because we aren't handling arrays.
                let reg_from = self.load_value_into_any(from);
                // We aren't actually writing to it, just giving it a new label. But then it might
                // be changed, so if from didn't get killed, we need to make sure it is saved for
                // later.
                self.meta_kill_variables(&instruction.kills);
                self.prepare_locked_register(reg_from);
                self.meta_unlock_registers();
                self.commit_value_in_register(to, reg_from);
            }
            i::Instruction::BinaryOperation { op, a, b, x } => {
                // TODO: Optimize.
                match op {
                    i::BinaryOperator::AddI => {
                        let reg_a = self.load_value_into_any(a);
                        let reg_b = self.load_value_into_any(b);
                        self.meta_kill_variables(&instruction.kills);
                        self.prepare_locked_register(reg_a);
                        self.write_byte(0x01); // Add to second operand.
                        self.write_modrm_two_register(reg_b, reg_a);
                        self.meta_unlock_registers();
                        self.commit_value_in_register(x, reg_a);
                    }
                    i::BinaryOperator::SubI => {
                        let reg_a = self.load_value_into_any(a);
                        let reg_b = self.load_value_into_any(b);
                        self.meta_kill_variables(&instruction.kills);
                        self.prepare_locked_register(reg_a);
                        self.write_byte(0x29); // Subtract from second operand.
                        self.write_modrm_two_register(reg_b, reg_a);
                        self.meta_unlock_registers();
                        self.commit_value_in_register(x, reg_a);
                    }
                    i::BinaryOperator::MulI => {
                        let reg_a = self.load_value_into_any(a);
                        let reg_b = self.load_value_into_any(b);
                        self.meta_kill_variables(&instruction.kills);
                        self.prepare_locked_register(reg_a);
                        self.write_bytes(&[0x0f, 0xaf]); // Multiply into first operand.
                        self.write_modrm_two_register(reg_a, reg_b);
                        self.meta_unlock_registers();
                        self.commit_value_in_register(x, reg_a);
                    }
                    i::BinaryOperator::DivI => {
                        // The divide instruction requires the first operand to be contained in the
                        // a and d registers, with d containing the high bits and a containing the
                        // low bits.
                        let reg_a = self.load_value_into_register(a, Register::A);
                        self.load_zero_into_register(Register::D);
                        let reg_b = self.load_value_into_any(b);
                        self.meta_kill_variables(&instruction.kills);
                        self.prepare_locked_register(reg_a);
                        // Divide by second operand, first operand is ignored.
                        self.write_bytes(&[0xF7]);
                        self.write_byte(0b11111000 | reg_b.opcode_index_u8());
                        self.meta_unlock_registers();
                        self.commit_value_in_register(x, reg_a);
                    }
                    i::BinaryOperator::ModI => {
                        // Same logic as above, because the divide instruction stores the remainder
                        // in the D register.
                        self.load_value_into_register(a, Register::A);
                        let reg_remainder = self.load_zero_into_register(Register::D);
                        let reg_b = self.load_value_into_any(b);
                        self.meta_kill_variables(&instruction.kills);
                        self.prepare_locked_register(reg_remainder);
                        // Divide by second operand, first operand is ignored.
                        self.write_bytes(&[0xF7]);
                        self.write_byte(0b11111000 | reg_b.opcode_index_u8());
                        self.meta_unlock_registers();
                        self.commit_value_in_register(x, reg_remainder);
                    }
                    i::BinaryOperator::AddF => {
                        let reg_a = self.load_value_into_any(a);
                        let reg_b = self.load_value_into_any(b);
                        self.meta_kill_variables(&instruction.kills);
                        self.prepare_locked_register(reg_a);
                        self.write_bytes(&[0xf3, 0x0f, 0x58]); // Addss to first operand.
                        self.write_modrm_two_register(reg_a, reg_b);
                        self.meta_unlock_registers();
                        self.commit_value_in_register(x, reg_a);
                    }
                    i::BinaryOperator::SubF => {
                        let reg_a = self.load_value_into_any(a);
                        let reg_b = self.load_value_into_any(b);
                        self.meta_kill_variables(&instruction.kills);
                        self.prepare_locked_register(reg_a);
                        self.write_bytes(&[0xf3, 0x0f, 0x5c]); // Subss from the first operand.
                        self.write_modrm_two_register(reg_a, reg_b);
                        self.meta_unlock_registers();
                        self.commit_value_in_register(x, reg_a);
                    }
                    i::BinaryOperator::MulF => {
                        let reg_a = self.load_value_into_any(a);
                        let reg_b = self.load_value_into_any(b);
                        self.meta_kill_variables(&instruction.kills);
                        self.prepare_locked_register(reg_a);
                        self.write_bytes(&[0xf3, 0x0f, 0x59]); // Mulss the first operand.
                        self.write_modrm_two_register(reg_a, reg_b);
                        self.meta_unlock_registers();
                        self.commit_value_in_register(x, reg_a);
                    }
                    i::BinaryOperator::DivF => {
                        let reg_a = self.load_value_into_any(a);
                        let reg_b = self.load_value_into_any(b);
                        self.meta_kill_variables(&instruction.kills);
                        self.prepare_locked_register(reg_a);
                        self.write_bytes(&[0xf3, 0x0f, 0x5e]); // Divss the first operand.
                        self.write_modrm_two_register(reg_a, reg_b);
                        self.meta_unlock_registers();
                        self.commit_value_in_register(x, reg_a);
                    }
                    i::BinaryOperator::ModF => {
                        // There is no xmm modulo instruction, so we have to do a messy chain of
                        // instructions instead.
                        let reg_a = self.load_value_into_any(a);
                        let reg_b = self.load_value_into_any(b);
                        // We need to save the value of a for later.
                        let reg_a2 = self.prepare_any_register_for(VariableType::F32);
                        self.write_mov_reg32_to_reg32(reg_a, reg_a2);

                        self.meta_kill_variables(&instruction.kills);
                        self.prepare_locked_register(reg_a);
                        self.write_bytes(&[0xf3, 0x0f, 0x5e]); // Divss the first operand.
                        self.write_modrm_two_register(reg_a, reg_b);
                        // reg_a = a / b, reg_b = b, reg_a2 = a
                        // Convert 4xf32 from second operand to 4xi32 in first operand.
                        // Truncate ss in second operand to ss in first operand.
                        self.write_bytes(&[0x66, 0x0f, 0x3a, 0x0a]);
                        self.write_modrm_two_register(reg_a, reg_a);
                        self.write_byte(0b0011); // Specify rounding mode should be trucation.
                                                 // reg_a = trunc(a / b), reg_b = b, reg_a2 = a
                        self.write_bytes(&[0xf3, 0x0f, 0x59]); // Mulss first operand.
                        self.write_modrm_two_register(reg_a, reg_b);
                        // reg_a = b * trunc(a / b), reg_b = b, reg_a2 = a
                        self.write_bytes(&[0xf3, 0x0f, 0x5c]); // Subss from the first operand.
                        self.write_modrm_two_register(reg_a2, reg_a);
                        // reg_a = b * trunc(a / b), reg_b = b, reg_a2 = a % b

                        self.meta_unlock_registers();
                        self.commit_value_in_register(x, reg_a2);
                    }
                    _ => unimplemented!("{:?}", op),
                }
            }
            i::Instruction::Compare { a, b } => {
                let reg_a = self.load_value_into_any(a);
                let reg_b = self.load_value_into_any(b);
                self.meta_kill_variables(&instruction.kills);
                self.write_byte(0x39); // Compare two registers.
                self.write_modrm_two_register(reg_a, reg_b);
                self.meta_unlock_registers();
            }
            i::Instruction::Assert(condition) => {
                self.meta_kill_variables(&instruction.kills);
                // Skip over the following pieces of code if the condition was true.
                self.write_jmp_cond_disp8(condition.code(), 10);
                // Return a value of 1 to indicate failure.
                self.write_mov_imm32_to_register(Register::A, 1); // 5 bytes
                self.write_restore_rsp_and_return(); // 5 bytes
                self.meta_unlock_registers();
            }
        }
    }
}
