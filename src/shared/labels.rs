use std::fmt::{self, Debug, Formatter};

#[derive(Clone, Copy, Eq, Hash, PartialEq)]
pub struct LabelId(usize);

impl Debug for LabelId {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        write!(formatter, "l{}", self.0)
    }
}

pub struct Label {
    single_source: bool,
}

pub struct LabelStorage(Vec<Label>);

impl Debug for LabelStorage {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        write!(formatter, "{} labels", self.0.len())
    }
}

impl LabelStorage {
    pub fn new() -> LabelStorage {
        LabelStorage(Vec::new())
    }

    pub fn create_label(&mut self, single_source: bool) -> LabelId {
        let id = LabelId(self.0.len());
        self.0.push(Label { single_source });
        id
    }

    pub fn is_label_single_source(&mut self, label: LabelId) -> bool {
        assert!(label.0 < self.0.len());
        self.0[label.0].single_source
    }
}
