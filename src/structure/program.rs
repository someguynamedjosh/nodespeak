use crate::structure::{
    add_builtins, make_var, Builtins, DataType, Entity, FuncCall, FunctionEntity, KnownData, Scope,
};
use std::collections::HashMap;

/// Refers to a [`Scope`] stored in a [`Program`].
///
/// You'll notice that this struct requires no lifetime. This was chosen to allow for easy
/// implementation of tree-like and cyclic data structures inside the library.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct ScopeId {
    raw_id: usize,
}

impl ScopeId {
    fn new(raw_id: usize) -> ScopeId {
        ScopeId { raw_id: raw_id }
    }

    fn get_raw(&self) -> usize {
        self.raw_id
    }
}

/// Refers to an [`Entity`] stored in a [`Program`].
///
/// You'll notice that this struct requires no lifetime. This was chosen to allow for easy
/// implementation of tree-like and cyclic data structures inside the library.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct EntityId {
    raw_id: usize,
}

impl EntityId {
    fn new(raw_id: usize) -> EntityId {
        EntityId { raw_id: raw_id }
    }

    fn get_raw(&self) -> usize {
        self.raw_id
    }
}

#[derive(Clone, Debug)]
struct EntityData {
    initial: KnownData,
    permanent: bool,
    temporary: KnownData,
}

#[derive(Debug)]
/// Represents an entire program written in the Waveguide language.
pub struct Program {
    scopes: Vec<Scope>,
    root_scope: ScopeId,
    entities: Vec<Entity>,
    entity_data: Vec<EntityData>,
    builtins: Option<Builtins>,
}

impl Program {
    pub fn new() -> Program {
        let mut prog = Program {
            scopes: vec![Scope::new()],
            root_scope: ScopeId::new(0),
            entities: Vec::new(),
            entity_data: Vec::new(),
            builtins: Option::None,
        };
        prog.builtins = Option::Some(add_builtins(&mut prog));
        prog
    }

    // ===SCOPES====================================================================================

    /// Creates a new scope that has no parent.
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

    pub fn get_scope_parent(&self, child: ScopeId) -> Option<ScopeId> {
        assert!(child.get_raw() < self.scopes.len());
        self.scopes[child.get_raw()].parent
    }

    pub fn add_func_call(&mut self, add_to: ScopeId, call: FuncCall) {
        assert!(add_to.get_raw() < self.scopes.len());
        self.scopes[add_to.get_raw()].body.push(call);
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

    pub fn clone_scope_body(&self, scope: ScopeId) -> Vec<FuncCall> {
        assert!(scope.get_raw() < self.scopes.len());
        self.scopes[scope.get_raw()].body.clone()
    }

    pub fn iterate_over_scope_body(&self, scope: ScopeId) -> std::slice::Iter<FuncCall> {
        assert!(scope.get_raw() < self.scopes.len());
        self.scopes[scope.get_raw()].body.iter()
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

    // ===SYMBOLS/ENTITIES==========================================================================

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

    pub fn modify_entity(&mut self, entity: EntityId, modified: Entity) {
        assert!(entity.get_raw() < self.entities.len());
        self.entities[entity.get_raw()] = modified;
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

    fn create_data_for(entity: &Entity) -> EntityData {
        EntityData {
            initial: match entity {
                Entity::IntLiteral(value) => KnownData::Int(*value),
                Entity::FloatLiteral(value) => KnownData::Float(*value),
                Entity::BoolLiteral(value) => KnownData::Bool(*value),
                // TODO: Initial values for variables.
                _ => KnownData::Unknown,
            },
            permanent: match entity {
                Entity::IntLiteral(_value) => true,
                Entity::FloatLiteral(_value) => true,
                Entity::BoolLiteral(_value) => true,
                _ => false
            },
            temporary: KnownData::Unknown
        }
    }

    pub fn adopt_entity(&mut self, entity: Entity) -> EntityId {
        let id = EntityId::new(self.entities.len());
        self.entity_data.push(Self::create_data_for(&entity));
        self.entities.push(entity);
        id
    }

    pub fn borrow_entity(&self, entity: EntityId) -> &Entity {
        assert!(entity.get_raw() < self.entities.len());
        &self.entities[entity.get_raw()]
    }

    // ===DATA STORAGE==============================================================================

    /// Sets the value that the specified entity is known to have at the start of the program.
    ///
    /// By default, all variables are given appropriate default values for their data types. Literal
    /// entities such as [`Entity::IntLiteral`] are set to whatever the value of the literal is.
    pub fn set_initial_data(&mut self, entity: EntityId, data: KnownData) {
        assert!(entity.get_raw() < self.entities.len());
        self.entity_data[entity.get_raw()].initial = data;
    }

    /// Gets the value that the specified entity is known to have at the start of the program.
    ///
    /// By default, all variables are given appropriate default values for their data types. Literal
    /// entities such as [`Entity::IntLiteral`] are set to whatever the value of the literal is.
    pub fn get_initial_data(&self, entity: EntityId) -> &KnownData {
        assert!(entity.get_raw() < self.entities.len());
        &self.entity_data[entity.get_raw()].initial
    }

    /// Marks the specified entity as having a value that remains unchanged from its initial value
    /// throughout the entire program.
    pub fn mark_initial_data_as_permanent(&mut self, entity: EntityId) {
        assert!(entity.get_raw() < self.entities.len());
        self.entity_data[entity.get_raw()].permanent = true;
    }

    /// Checks if the data of the specified entity remains unchanged from the initial value 
    /// throughout the entire program.
    pub fn is_initial_data_permanent(&mut self, entity: EntityId) -> bool {
        assert!(entity.get_raw() < self.entities.len());
        self.entity_data[entity.get_raw()].permanent
    }

    /// Stores a temporary value for the related entity.
    ///
    /// This is used during interpretation and simplification to keep track of the current value
    /// of a variable that does not have a permanent value across the entire program.
    pub fn set_temporary_data(&mut self, entity: EntityId, data: KnownData) {
        assert!(entity.get_raw() < self.entities.len());
        self.entity_data[entity.get_raw()].temporary = data;
    }

    /// Retrieves a temporary value for the related entity, set by set_temporary_data.
    ///
    /// This is used during interpretation and simplification to keep track of the current value
    /// of a variable that does not have a permanent value across the entire program.
    pub fn get_temporary_data(&self, entity: EntityId) -> &KnownData {
        assert!(entity.get_raw() < self.entities.len());
        &self.entity_data[entity.get_raw()].temporary
    }

    /// Resets the temporary data of an entity to be the permanent data of that entity.
    ///
    /// Equivalent to calling `set_temporary_data(entity, get_permanent_data(entity))
    pub fn reset_temporary_data(&mut self, entity: EntityId) {
        assert!(entity.get_raw() < self.entities.len());
        let data = &mut self.entity_data[entity.get_raw()];
        data.temporary = data.initial.clone();
    }

    /// Resets the temporary data of every entity to their permanent data values.
    ///
    /// This should be used before beginning interpretation or simplification to reset all values
    /// to a value they are known to have at the beginning of the program.
    pub fn reset_all_temporary_data(&mut self) {
        for index in 0..self.entity_data.len() {
            let data = &mut self.entity_data[index];
            data.temporary = data.initial.clone();
        }
    }

    // ===UTILITIES=================================================================================

    pub fn get_root_scope(&self) -> ScopeId {
        self.root_scope
    }

    pub fn set_root_scope(&mut self, new_root_scope: ScopeId) {
        self.root_scope = new_root_scope;
    }

    pub fn get_builtins(&self) -> &Builtins {
        self.builtins.as_ref().unwrap()
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

    // Tries to find the smallest function that contains the specified scope.
    // If there is none (e.g. the scope provided is the root), Option::None is
    // returned.
    pub fn find_parent_function(&self, scope: ScopeId) -> Option<EntityId> {
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
}
