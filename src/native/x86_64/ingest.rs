use crate::trivial::structure as i;
use super::Program;

pub fn ingest(program: &i::Program) -> Program {
    let mut assembler = Assembler::new(program);
    assembler.entry_point();
    assembler.finalize()
}

struct Assembler<'a> {
    source: &'a i::Program
}

impl<'a> Assembler<'a> {
    fn new(source: &i::Program) -> Assembler {
        Assembler {
            source
        }
    }

    fn entry_point(&mut self) {
        unimplemented!()
    }

    fn finalize(&mut self) -> Program {
        unimplemented!()
    }
}