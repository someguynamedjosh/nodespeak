use crate::vague::{
    biggest_common_type, make_var, DataType, Entity, EntityId, FunctionEntity, Program, ScopeId,
};
use std::collections::HashMap;
// This file intended to convert scopes containing automatic / template variables
// into scopes which have only specific data types. Eventually, it will also
// use the interpreter to compute as much of the program as it can at compile
// time.

pub fn resolve_scope(program: &mut Program, scope: ScopeId) -> ScopeId {
    let mut resolver = ScopeResolver::new(program);
    let result = resolver.resolve(scope);
    drop(resolver);
    result
}

struct ScopeResolver<'a> {
    program: &'a mut Program,
    // Since entity IDs are global, we don't need to worry about keeping a separate
    // table for each scope, IDs from separate scopes do not overlap.
    conversion_table: HashMap<EntityId, EntityId>,
}

impl<'a> ScopeResolver<'a> {
    fn new(program: &'a mut Program) -> ScopeResolver<'a> {
        ScopeResolver {
            program: program,
            conversion_table: HashMap::new(),
        }
    }

    fn add_conversion(&mut self, from: EntityId, to: EntityId) {
        assert!(
            !self.conversion_table.contains_key(&from),
            "Cannot have multiple conversions for a single entity!"
        );
        self.conversion_table.insert(from, to);
    }

    fn initial_copy(&mut self, source: ScopeId) -> ScopeId {
        let copy = match self.program.get_scope_parent(source) {
            Option::Some(parent_id) => self.program.create_child_scope(parent_id),
            Option::None => self.program.create_scope(),
        };

        // TODO: We probably don't need to preserve the variable names in the
        // resolved scope. Depends on how some meta features get implemented in
        // the future.
        let symbol_table = self.program.clone_symbols_in(source);
        for name_value_pair in symbol_table.iter() {
            let id = *name_value_pair.1;
            match self.program.borrow_entity(id.clone()) {
                // If it is a variable, we want to check if it has an automatic
                // data type. If not, we can just redefine the variable in the
                // new scope without creating a new entity because we will never
                // need to perform any modification on it.
                Entity::Variable(data) => {
                    let typ = self.program.get_entity_data_type(id.clone());
                    if typ.is_automatic() {
                        // We need to make a new definition of the data type so
                        // that we can use that data type only for this one
                        // variable, and then change the data type to what it
                        // should actually be once we find out, and not have to
                        // worry about changing any IDs pointing to it.
                        let type_id = self
                            .program
                            .adopt_and_define_intermediate(copy, Entity::DataType(typ));
                        let copied_var = self.program.adopt_and_define_symbol(
                            copy,
                            &*name_value_pair.0,
                            make_var(type_id),
                        );
                        self.add_conversion(id, copied_var);
                    } else {
                        self.program.define_symbol(copy, &*name_value_pair.0, id);
                    }
                }
                // Just copy it for now. Conversion only needs to be done when
                // the function is called.
                Entity::Function(data) => {
                    self.program.define_symbol(copy, &*name_value_pair.0, id);
                }
                // There's nothing to convert here.
                Entity::BuiltinFunction(data) => {
                    self.program.define_symbol(copy, &*name_value_pair.0, id);
                }
                Entity::BoolLiteral(value) => {
                    self.program.define_symbol(copy, &*name_value_pair.0, id);
                }
                Entity::IntLiteral(value) => {
                    self.program.define_symbol(copy, &*name_value_pair.0, id);
                }
                Entity::FloatLiteral(value) => {
                    self.program.define_symbol(copy, &*name_value_pair.0, id);
                }
                // I think this can just be copied. Might be wrong though, in which case, hello me
                // from the future! Sorry you had to spend time debugging this.
                Entity::DataType(data) => {
                    self.program.define_symbol(copy, &*name_value_pair.0, id);
                }
                _ => unimplemented!(),
            }
        }

        copy
    }

    fn resolve(&mut self, target: ScopeId) -> ScopeId {
        let copy = self.initial_copy(target);
        copy
    }
}

// Takes in an unresolved scope and creates a new, resolved version.
pub fn old_resolve_scope(program: &mut Program, scope: ScopeId) -> ScopeId {
    let has_parent;
    let resolved = match program.get_scope_parent(scope) {
        Option::Some(parent) => {
            has_parent = true;
            program.create_child_scope(parent)
        }
        Option::None => {
            has_parent = false;
            program.create_scope()
        }
    };

    // Check each function call to convert its template parameters, and use the
    // resolved signature to resolve any automatic variables used as inputs or outputs.
    for func_call in program.iterate_over_scope_body(scope) {
        let target;
        // Get the FunctionEntity the function call is targeting.
        match program.borrow_entity(func_call.get_function()) {
            Entity::Variable(_data) => unimplemented!(),
            Entity::Function(data) => target = data.clone(),
            Entity::BuiltinFunction(data) => target = data.get_base(),
            _ => unreachable!(),
        }
        // Collect the EntityId of any data types used as template parameters.
        let mut type_parameters = Vec::new();
        for template_parameter_id in target.iterate_over_template_parameters() {
            match program.borrow_entity(template_parameter_id.clone()) {
                Entity::DataType(_type) => type_parameters.push(template_parameter_id.clone()),
                _ => (),
            }
        }
        // Make a vector of real types, one for each type template parameter.
        let mut real_types = Vec::new();
        for _ in 0..type_parameters.len() {
            // We don't know what the real types will be yet.
            real_types.push(Option::<DataType>::None);
        }
        // Iterate over all the inputs to the function call.
        let mut index = 0usize.wrapping_sub(1);
        for input in func_call.iterate_over_inputs() {
            index = index.wrapping_add(1);
            let target_input = target.get_input(index);
            let mut matching_type_index = 0xFFFF;
            match program.borrow_entity(target_input) {
                Entity::Variable(data) => {
                    // Get the actual data type that the original function signature requests.
                    for type_param_index in 0..type_parameters.len() {
                        // Note if the data type the function is requesting is a template param.
                        if data.get_data_type() == type_parameters[type_param_index] {
                            matching_type_index = type_param_index;
                            break;
                        }
                    }
                }
                _ => unreachable!(),
            }
            // If we didn't find anything, then no more processing is necessary
            // for this input.
            if matching_type_index == 0xFFFF {
                continue;
            }
            // Otherwise, find the data type of the value given in the function
            // call and add it as a possibility.
            let possibility = program.get_entity_data_type(input.get_base());
            match &real_types[matching_type_index] {
                Option::None => real_types[matching_type_index] = Option::Some(possibility),
                Option::Some(other_possibility) => {
                    real_types[matching_type_index] =
                        Option::Some(biggest_common_type(&possibility, other_possibility))
                }
            }
        }
    }

    // Normally functions get resolved because they are called elsewhere in the
    // code. But main is only called by code outside what we parse, so we need
    // to specifically check for a main function and convert it. Specifically,
    // we only want to convert a function named 'main' in a scope that has no
    // parent, making it the root scope.
    if !has_parent {
        match program.shallow_lookup_symbol(scope, "main") {
            Option::Some(id) => {
                match program.borrow_entity(id) {
                    Entity::Function(dat) => {
                        let data = dat.clone();
                        let new_scope = resolve_scope(program, data.get_body());
                        let mut new_function = FunctionEntity::new(new_scope);
                        // Main function should not have any template parameters. For now, just copy
                        // data as if they do not exist. TODO: Error message for putting template
                        // parameters in main function.
                        for index in 0..data.get_num_inputs() {
                            new_function.add_input(data.get_input(index));
                        }
                        for index in 0..data.get_num_outputs() {
                            new_function.add_output(data.get_output(index));
                        }
                    }
                    _ => unreachable!(),
                }
            }
            Option::None => (),
        }
    }

    resolved
}
