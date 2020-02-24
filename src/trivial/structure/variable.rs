use std::fmt::{self, Debug, Formatter};
use crate::shared::NativeType;

#[derive(Clone)]
pub struct Variable {
    typ: NativeType,
}

impl Debug for Variable {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        write!(formatter, "{:?}", self.typ)
    }
}

impl Variable {
    pub fn new(typ: NativeType) -> Variable {
        Variable {
            typ,
        }
    }

    pub fn borrow_type(&self) -> &NativeType {
        &self.typ
    }

    pub fn borrow_dimensions(&self) -> &Vec<usize> {
        self.typ.borrow_dimensions()
    }

    pub fn get_physical_size(&self) -> usize {
        self.typ.get_physical_size()
    }
}
