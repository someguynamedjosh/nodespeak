use crate::structure::{
    add_builtins, make_var, Builtins, DataType, Entity, FuncCall, FunctionEntity, KnownData, Scope,
};
use std::collections::HashMap;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
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

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
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
    entity_data: Vec<KnownData>,
    builtins: Option<Builtins>,
}

impl Program {
    pub fn new() -> Program {
        let mut prog = Program {
            scopes: vec![Scope::new()],
            entities: Vec::new(),
            entity_data: Vec::new(),
            builtins: Option::None,
        };
        prog.builtins = Option::Some(add_builtins(&mut prog));
        prog
    }

    pub fn get_builtins(&self) -> &Builtins {
        self.builtins.as_ref().unwrap()
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

    pub fn get_scope_parent(&self, child: ScopeId) -> Option<ScopeId> {
        assert!(child.get_raw() < self.scopes.len());
        self.scopes[child.get_raw()].parent
    }

    pub fn add_func_call(&mut self, add_to: ScopeId, call: FuncCall) {
        assert!(add_to.get_raw() < self.scopes.len());
        self.scopes[add_to.get_raw()].body.push(call);
    }

    pub fn clone_scope_body(&self, scope: ScopeId) -> Vec<FuncCall> {
        assert!(scope.get_raw() < self.scopes.len());
        self.scopes[scope.get_raw()].body.clone()
    }

    pub fn iterate_over_scope_body(&self, scope: ScopeId) -> std::slice::Iter<FuncCall> {
        assert!(scope.get_raw() < self.scopes.len());
        self.scopes[scope.get_raw()].body.iter()
    }

    pub fn shallow_lookup_symbol(&self, scope: ScopeId, symbol: &str) -> Option<EntityId> {
        assert!(scope.get_raw() < self.scopes.len());
        let real_scope = &self.scopes[scope.get_raw()];
        match real_scope.symbols.get(symbol) {
            Option::Some(value) => Option::Some(*value),
            Option::None => Option::None,
        }
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

    // Tries to find the smallest function that contains the specified scope.
    // If there is none (e.g. the scope provided is the root), Option::None is
    // returned.
    pub fn lookup_parent_function(&self, scope: ScopeId) -> Option<EntityId> {
        // TODO: This is very inefficient.
        let mut real_scope = scope.get_raw();
        loop {
            let mut index: usize = 0;
            for entity in self.entities.iter() {
                if let Entity::Function(function_entity) = entity {
                    if function_entity.get_body().get_raw() == real_scope {
                        return Option::Some(EntityId::new(index));
                    }
                }
                index += 1;
            }
            match self.scopes[real_scope].parent {
                Option::None => return Option::None,
                Option::Some(id) => real_scope = id.get_raw(),
            };
        }
    }

    // Tries to find the smallest function that contains the specified scope.
    // If there is none (e.g. the scope provided is the root), Option::None is
    // returned.
    pub fn lookup_and_clone_parent_function(&self, scope: ScopeId) -> Option<FunctionEntity> {
        // TODO: This is very inefficient.
        let mut real_scope = scope.get_raw();
        loop {
            for entity in self.entities.iter() {
                if let Entity::Function(function_entity) = entity {
                    if function_entity.get_body().get_raw() == real_scope {
                        return Option::Some(function_entity.clone());
                    }
                }
            }
            match self.scopes[real_scope].parent {
                Option::None => return Option::None,
                Option::Some(id) => real_scope = id.get_raw(),
            };
        }
    }

    pub fn define_symbol(&mut self, scope: ScopeId, symbol: &str, definition: EntityId) {
        assert!(definition.get_raw() < self.entities.len());
        assert!(scope.get_raw() < self.scopes.len());
        self.scopes[scope.get_raw()]
            .symbols
            .insert(symbol.to_owned(), definition);
    }

    pub fn define_intermediate(&mut self, scope: ScopeId, definition: EntityId) {
        assert!(definition.get_raw() < self.entities.len());
        assert!(scope.get_raw() < self.scopes.len());
        self.scopes[scope.get_raw()].intermediates.push(definition);
    }

    pub fn redefine_entity(&mut self, old_entity: EntityId, new_definition: Entity) {
        assert!(old_entity.get_raw() < self.entities.len());
        self.entities[old_entity.get_raw()] = new_definition;
    }

    pub fn iterate_over_entities_defined_by(
        &self,
        scope: ScopeId,
    ) -> std::iter::Chain<
        std::collections::hash_map::Values<String, EntityId>,
        std::slice::Iter<EntityId>,
    > {
        assert!(scope.get_raw() < self.scopes.len());
        self.scopes[scope.get_raw()]
            .symbols
            .values()
            .chain(self.scopes[scope.get_raw()].intermediates.iter())
    }

    pub fn iterate_over_symbols_in(
        &self,
        scope: ScopeId,
    ) -> std::collections::hash_map::Iter<String, EntityId> {
        assert!(scope.get_raw() < self.scopes.len());
        self.scopes[scope.get_raw()].symbols.iter()
    }

    pub fn clone_symbols_in(&self, scope: ScopeId) -> HashMap<String, EntityId> {
        assert!(scope.get_raw() < self.scopes.len());
        self.scopes[scope.get_raw()].symbols.clone()
    }

    pub fn iterate_over_intermediates_in(&self, scope: ScopeId) -> std::slice::Iter<EntityId> {
        assert!(scope.get_raw() < self.scopes.len());
        self.scopes[scope.get_raw()].intermediates.iter()
    }

    pub fn clone_intermediates_in(&self, scope: ScopeId) -> Vec<EntityId> {
        assert!(scope.get_raw() < self.scopes.len());
        self.scopes[scope.get_raw()].intermediates.clone()
    }

    fn create_data(entity: &Entity) -> KnownData {
        match entity {
            Entity::IntLiteral(value) => KnownData::Int(*value),
            Entity::FloatLiteral(value) => KnownData::Float(*value),
            Entity::BoolLiteral(value) => KnownData::Bool(*value),
            _ => KnownData::Empty,
        }
    }

    pub fn adopt_entity(&mut self, entity: Entity) -> EntityId {
        let id = EntityId::new(self.entities.len());
        self.entity_data.push(Self::create_data(&entity));
        self.entities.push(entity);
        id
    }

    pub fn modify_entity(&mut self, entity: EntityId, modified: Entity) {
        assert!(entity.get_raw() < self.entities.len());
        self.entity_data[entity.get_raw()] = Self::create_data(&modified);
        self.entities[entity.get_raw()] = modified;
    }

    pub fn borrow_entity(&self, entity: EntityId) -> &Entity {
        assert!(entity.get_raw() < self.entities.len());
        &self.entities[entity.get_raw()]
    }

    pub fn get_entity_data(&self, entity: EntityId) -> &KnownData {
        assert!(entity.get_raw() < self.entities.len());
        &self.entity_data[entity.get_raw()]
    }

    pub fn set_entity_data(&mut self, entity: EntityId, data: KnownData) {
        assert!(entity.get_raw() < self.entities.len());
        self.entity_data[entity.get_raw()] = data;
    }

    pub fn get_entity_data_type(&self, entity: EntityId) -> DataType {
        assert!(entity.get_raw() < self.entities.len());
        match &self.entities[entity.get_raw()] {
            Entity::BoolLiteral(_value) => DataType::Bool,
            Entity::IntLiteral(_value) => DataType::Int,
            Entity::FloatLiteral(_value) => DataType::Float,
            Entity::Variable(var) => {
                let data_type_entity = &self.entities[var.get_data_type().get_raw()];
                match data_type_entity {
                    Entity::DataType(data_type) => data_type.clone(),
                    _ => unreachable!(),
                }
            }
            _ => DataType::Void,
        }
    }

    pub fn adopt_and_define_symbol(
        &mut self,
        scope: ScopeId,
        symbol: &str,
        definition: Entity,
    ) -> EntityId {
        let id = self.adopt_entity(definition);
        self.define_symbol(scope, symbol, id);
        id
    }

    pub fn adopt_and_define_intermediate(
        &mut self,
        scope: ScopeId,
        definition: Entity,
    ) -> EntityId {
        let id = self.adopt_entity(definition);
        self.define_intermediate(scope, id);
        id
    }

    pub fn make_intermediate_auto_var(&mut self, scope: ScopeId) -> EntityId {
        self.adopt_and_define_intermediate(scope, make_var(self.get_builtins().automatic_type))
    }
}
