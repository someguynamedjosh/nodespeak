use crate::vague::{Entity, Scope};

#[derive(Clone, Copy, Debug)]
pub struct ScopeId {
    raw_id: usize,
}

impl ScopeId {
    pub fn new(raw_id: usize) -> ScopeId {
        ScopeId { raw_id: raw_id }
    }

    pub fn get_raw(&self) -> usize {
        self.raw_id
    }
}

#[derive(Clone, Copy, Debug)]
pub struct EntityId {
    raw_id: usize,
}

impl EntityId {
    pub fn new(raw_id: usize) -> EntityId {
        EntityId { raw_id: raw_id }
    }

    pub fn get_raw(&self) -> usize {
        self.raw_id
    }
}

#[derive(Debug)]
pub struct Program {
    scopes: Vec<Scope>,
    entities: Vec<Entity>,
}

impl Program {
    pub fn new() -> Program {
        Program {
            scopes: vec![Scope::new()],
            entities: Vec::new(),
        }
    }

    pub fn create_scope(&mut self) -> ScopeId {
        let id = ScopeId::new(self.scopes.len());
        self.scopes.push(Scope::new());
        id
    }

    pub fn create_child_scope(&mut self, parent: ScopeId) -> ScopeId {
        assert!(parent.get_raw() < self.scopes.len());
        let id = ScopeId::new(self.scopes.len());
        self.scopes.push(Scope::from_parent(parent));
        id
    }

    pub fn get_root_scope(&self) -> ScopeId {
        // The constructor automatically creates a scope at index 0. This method
        // is the only way to access that ID, because all the other methods that
        // return ScopeIds also create the scopes their IDs are pointing to.
        ScopeId::new(0)
    }

    pub fn lookup_symbol(&self, scope: ScopeId, symbol: &str) -> Option<EntityId> {
        assert!(scope.get_raw() < self.scopes.len());
        let real_scope = &self.scopes[scope.get_raw()];
        let result = real_scope.symbols.get(symbol);
        match result {
            Option::Some(value_) => Option::Some(*value_),
            Option::None => match real_scope.parent {
                Option::Some(parent) => self.lookup_symbol(parent, symbol),
                Option::None => Option::None,
            },
        }
    }

    pub fn define_symbol(&mut self, scope: ScopeId, symbol: &str, definition: EntityId) {
        assert!(definition.get_raw() < self.entities.len());
        assert!(scope.get_raw() < self.scopes.len());
        self.scopes[scope.get_raw()]
            .symbols
            .insert(symbol.to_owned(), definition);
    }

    pub fn adopt_entity(&mut self, entity: Entity) -> EntityId {
        let id = EntityId::new(self.entities.len());
        self.entities.push(entity);
        id
    }
}
