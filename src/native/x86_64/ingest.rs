use std::io::Cursor;
use x86asm::{Instruction, InstructionWriter, Mnemonic, Mode, Operand, Reg};

use super::structure as o;
use crate::specialized::structure as i;

pub fn ingest(program: &i::Program) -> o::Program {
    let mut assembler = Assembler::new(program);
    assembler.entry_point();
    assembler.finalize()
}

struct Assembler<'a> {
    source: &'a i::Program,
    writer: InstructionWriter<Cursor<Vec<u8>>>,
}

impl<'a> Assembler<'a> {
    fn new(source: &i::Program) -> Assembler {
        let buffer = Cursor::new(Vec::new());
        let writer = InstructionWriter::new(buffer, Mode::Long);
        Assembler { source, writer }
    }

    fn entry_point(&mut self) {
        self.writer
            .write(&Instruction::new2(
                Mnemonic::MOV,
                Operand::Direct(Reg::RAX),
                Operand::Literal32(20),
            ))
            .unwrap();
    }

    fn finalize(&mut self) -> o::Program {
        let program = o::Program::new(0, 0);
        let code_vec = self.writer.get_inner_writer_ref().get_ref();
        println!("{} bytes of x64 code.", code_vec.len());
        program
    }
}
