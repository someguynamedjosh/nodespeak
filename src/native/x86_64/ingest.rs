use std::io::Cursor;

use super::structure as o;
use crate::specialized::structure as i;

pub fn ingest(program: &i::Program) -> o::Program {
    let mut assembler = Assembler::new(program);
    assembler.entry_point();
    assembler.finalize()
}

enum Register {
    A,
    C,
    D,
    B,
    SP,
    BP,
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
            Self::BP => 5,
            Self::SI => 6,
            Self::DI => 7,
        }
    }
}

struct Assembler<'a> {
    source: &'a i::Program,
    target: Vec<u8>,
}

impl<'a> Assembler<'a> {
    fn new(source: &i::Program) -> Assembler {
        Assembler { source, target: Vec::new() }
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

    // arg1: register, arg2: value of disp32.
    fn write_modrm_reg_disp32(&mut self, register: Register) {
        // 0b00 with 0b100 as the last part indicates 32-bit displacement without a base.
        self.write_byte(0b11000100 | register.index() << 3);
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

    fn mov_imm32_to_register(&mut self, register: Register, imm32: u32) {
        self.write_byte(0xb8 + register.index());
        self.write_imm32(imm32);
    }

    fn ret(&mut self) {
        self.write_byte(0xc3);
    }

    fn entry_point(&mut self) {
        self.mov_imm32_to_register(Register::A, 0x01234567);
        self.ret();
    }

    fn finalize(&mut self) -> o::Program {
        let mut program = o::Program::new(self.target.len(), 0);
        program.write_iter_to_code(0, self.target.iter());
        unsafe {
            assert!(program.execute() == 0x01234567);
        }
        program
    }
}
