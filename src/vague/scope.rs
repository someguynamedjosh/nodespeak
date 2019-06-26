use crate::vague::Entity;
use std::collections::HashMap;

pub struct Scope<'a> {
    symbols: HashMap<String, Entity>,
    parent: Option<&'a Scope<'a>>
}

impl<'a> Scope<'a> {
    pub fn new() -> Scope<'a> {
        Scope {
            symbols: HashMap::new(),
            parent: Option::None
        }
    }

    pub fn from_parent(parent: &'a Scope<'a>) -> Scope<'a> {
        Scope {
            symbols: HashMap::new(),
            parent: Option::Some(parent)
        }
    }

    pub fn new_child(&self) -> Scope {
        Scope::from_parent(self)
    }

    pub fn lookup(&self, symbol: &str) -> Option<&Entity> {
        let result = self.symbols.get(symbol);
        match result {
            Option::None => match self.parent {
                Option::None => Option::None,
                Option::Some(parent) => parent.lookup(symbol),
            },
            Option::Some(_value) => result
        }
    }

    pub fn define(&mut self, symbol: &str, definition: Entity) {
        self.symbols.insert(symbol.to_owned(), definition);
    }
}