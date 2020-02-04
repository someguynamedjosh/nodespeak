use std::io::Cursor;

use super::structure as o;
use crate::specialized::structure as i;

pub fn ingest(program: &i::Program) -> o::Program {
    let mut assembler = Assembler::new(program);
    assembler.entry_point();
    assembler.finalize()
}

struct Assembler<'a> {
    source: &'a i::Program,
    target: Vec<u8>,
}

impl<'a> Assembler<'a> {
    fn new(source: &i::Program) -> Assembler {
        Assembler { source, target: Vec::new() }
    }

    fn entry_point(&mut self) {
        self.target.push(0);
    }

    fn finalize(&mut self) -> o::Program {
        let program = o::Program::new(self.target.len(), 0);
        println!("{} bytes of x64 code.", self.target.len());
        program
    }
}
