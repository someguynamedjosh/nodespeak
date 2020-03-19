use std::fmt::{self, Debug, Formatter};

#[derive(Clone, Copy, Eq, Hash, PartialEq)]
pub struct LabelId(usize);

impl Debug for LabelId {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        write!(formatter, "l{}", self.0)
    }
}

pub struct LabelCounter(usize);

impl Debug for LabelCounter {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        write!(formatter, "{} labels", self.0)
    }
}

impl LabelCounter {
    pub fn new() -> LabelCounter {
        LabelCounter(0)
    }

    pub fn create_label(&mut self) -> LabelId {
        let id = LabelId(self.0);
        self.0 += 1;
        id
    }
}