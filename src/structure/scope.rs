use crate::structure::{FuncCall, ScopeId, VariableId};
use std::collections::HashMap;

#[derive(Debug)]
pub struct Scope {
    pub symbols: HashMap<String, VariableId>,
    pub intermediates: Vec<VariableId>,
    pub body: Vec<FuncCall>,
    pub parent: Option<ScopeId>,
}

impl Scope {
    pub fn new() -> Scope {
        Scope {
            symbols: HashMap::new(),
            intermediates: Vec::new(),
            body: Vec::new(),
            parent: Option::None,
        }
    }

    pub fn from_parent(parent: ScopeId) -> Scope {
        Scope {
            symbols: HashMap::new(),
            intermediates: Vec::new(),
            body: Vec::new(),
            parent: Option::Some(parent),
        }
    }
}
