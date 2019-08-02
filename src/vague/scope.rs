use crate::vague::{EntityId, ScopeId};
use std::collections::HashMap;

#[derive(Debug)]
pub struct Scope {
    pub symbols: HashMap<String, EntityId>,
    pub parent: Option<ScopeId>,
}

impl Scope {
    pub fn new() -> Scope {
        Scope {
            symbols: HashMap::new(),
            parent: Option::None,
        }
    }

    pub fn from_parent(parent: ScopeId) -> Scope {
        Scope {
            symbols: HashMap::new(),
            parent: Option::Some(parent),
        }
    }
}
