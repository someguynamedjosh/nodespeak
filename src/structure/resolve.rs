use crate::structure::{
    biggest_common_type, make_var, DataType, Entity, EntityId, FuncCall, FunctionEntity, Program,
    ScopeId, VarAccess,
};
use std::collections::HashMap;
// This file intended to convert scopes containing automatic / template variables
// into scopes which have only specific data types. Eventually, it will also
// use the interpreter to compute as much of the program as it can at compile
// time.

pub fn resolve_scope(program: &mut Program, scope: ScopeId) -> ScopeId {
    let mut resolver = ScopeResolver::new(program);
    let result = resolver.entry_point(scope);
    drop(resolver);
    result
}

struct ScopeResolver<'a> {
    program: &'a mut Program,
    conversion_table: HashMap<EntityId, EntityId>,
    // Even though EntityIds are global (so we don't have to worry about id conflicts),
    // we still have to worry about a single entity having multiple conversions. For
    // example, type parameters can be resolved to different values depending on the
    // types used for the inputs and outputs of the function.
    conversion_stack: Vec<HashMap<EntityId, EntityId>>,
}

impl<'a> ScopeResolver<'a> {
    fn new(program: &'a mut Program) -> ScopeResolver<'a> {
        ScopeResolver {
            program: program,
            conversion_table: HashMap::new(),
            conversion_stack: Vec::new(),
        }
    }

    // Pushes the current state of the conversion table onto the stack. The state
    // can be restored with pop_table().
    fn push_table(&mut self) {
        self.conversion_stack.push(self.conversion_table.clone());
    }

    fn pop_table(&mut self) {
        self.conversion_table = self
            .conversion_stack
            .pop()
            .expect("Encountered extra unexpected stack pop");
    }

    fn add_conversion(&mut self, from: EntityId, to: EntityId) {
        assert!(
            !self.conversion_table.contains_key(&from),
            "Cannot have multiple conversions for a single entity!"
        );
        self.conversion_table.insert(from, to);
    }

    fn convert(&self, from: EntityId) -> EntityId {
        // Either the ID was remapped to something else, or the ID has remained
        // unchanged.
        *self.conversion_table.get(&from).unwrap_or(&from)
    }

    fn convert_var_access(&self, access: &VarAccess) -> VarAccess {
        let mut new_access = VarAccess::new(self.convert(access.get_base()));
        for index in access.iterate_over_indexes() {
            new_access.add_index(self.convert(*index));
        }
        new_access
    }

    fn convert_func_call(&self, call: &FuncCall) -> FuncCall {
        let mut result = FuncCall::new(self.convert(call.get_function()));
        for input in call.iterate_over_inputs() {
            result.add_input(self.convert_var_access(input));
        }
        for output in call.iterate_over_outputs() {
            result.add_output(self.convert_var_access(output));
        }
        result
    }

    fn initial_copy(&mut self, source: ScopeId, parent: Option<ScopeId>) -> ScopeId {
        let copy = match parent {
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
                        let type_id = data.get_data_type();
                        let copied_var = self.program.adopt_and_define_symbol(
                            copy,
                            &*name_value_pair.0,
                            // We don't need to clone the data type since we're just going to change
                            // it later anyway.
                            make_var(type_id),
                        );
                        self.add_conversion(id, copied_var);
                    } else {
                        let converted = self.convert(data.get_data_type());
                        if converted == data.get_data_type() {
                            // We can just copy it directly since the data type does not need to be
                            // converted.
                            self.program.define_symbol(copy, &*name_value_pair.0, id);
                        } else {
                            let copied_var = self.program.adopt_and_define_symbol(
                                copy,
                                &*name_value_pair.0,
                                // We don't need to clone the data type since we're just going to change
                                // it later anyway.
                                make_var(converted),
                            );
                            self.add_conversion(id, copied_var);
                        }
                    }
                }
                // Just copy it for now. Conversion only needs to be done when
                // the function is called.
                Entity::Function(_data) => {
                    self.program.define_symbol(copy, &*name_value_pair.0, id);
                }
                // There's nothing to convert here.
                Entity::BuiltinFunction(_data) => {
                    self.program.define_symbol(copy, &*name_value_pair.0, id);
                }
                Entity::BoolLiteral(_value) => {
                    self.program.define_symbol(copy, &*name_value_pair.0, id);
                }
                Entity::IntLiteral(_value) => {
                    self.program.define_symbol(copy, &*name_value_pair.0, id);
                }
                Entity::FloatLiteral(_value) => {
                    self.program.define_symbol(copy, &*name_value_pair.0, id);
                }
                // I think this can just be copied. Might be wrong though, in which case, hello me
                // from the future! Sorry you had to spend time debugging this.
                Entity::DataType(_data) => {
                    self.program.define_symbol(copy, &*name_value_pair.0, id);
                }
                _ => unimplemented!(),
            }
        }

        copy
    }

    fn resolve_automatic_type(automatic_type: &DataType, actual_type: &DataType) -> DataType {
        match automatic_type {
            DataType::Automatic => actual_type.clone(),
            DataType::Array(_specs) => {
                unimplemented!("Resolving complex automatic data types is unimplemented.")
            }
            _ => unreachable!("automatic_type should always be an automatic data type."),
        }
    }

    fn resolve_function_call(&mut self, old_func_call: &FuncCall, target_scope: ScopeId) {
        let mut func_call = self.convert_func_call(&old_func_call);
        let func_target;
        let is_builtin;
        // Get the FunctionEntity the function call is func_targeting.
        match self.program.borrow_entity(func_call.get_function()) {
            Entity::Variable(_data) => {
                unimplemented!("Calling a function variable is unimplemented.")
            }
            Entity::Function(data) => {
                func_target = data.clone();
                is_builtin = false;
            }
            Entity::BuiltinFunction(data) => {
                func_target = data.get_base();
                is_builtin = true;
            }
            _ => unreachable!(),
        }

        // Collect the EntityId of any data types used as template parameters.
        let mut type_parameters = Vec::new();
        let mut type_parameter_indexes = Vec::new();
        let mut index = 0usize;
        for template_parameter_id in func_target.iterate_over_template_parameters() {
            match self.program.borrow_entity(template_parameter_id.clone()) {
                Entity::DataType(_type) => {
                    type_parameters.push(template_parameter_id.clone());
                    type_parameter_indexes.push(index);
                }
                _ => (),
            }
            index += 1;
        }

        // Make a vector of real types, one for each type template parameter.
        let mut real_types = Vec::new();
        for _ in 0..type_parameters.len() {
            // We don't know what the real types will be yet.
            real_types.push(Option::<DataType>::None);
        }

        // Tries to find the index of the type of the specified function parameter
        // in the type_parameters vector.
        let get_type_index = |program: &Program, parameter: EntityId| -> Option<usize> {
            match program.borrow_entity(parameter) {
                Entity::Variable(data) => {
                    // Get the actual data type that the original function signature requests.
                    for type_param_index in 0..type_parameters.len() {
                        // Note if the data type the function is requesting is a template param.
                        if data.get_data_type() == type_parameters[type_param_index] {
                            return Some(type_param_index);
                        }
                    }
                }
                _ => unreachable!(),
            }
            return None;
        };

        // Iterate over all the inputs to the function call.
        for (index, input) in func_call.iterate_over_inputs().enumerate() {
            let func_target_input = func_target.get_input(index);
            match get_type_index(self.program, func_target_input) {
                Option::None => {
                    // If we didn't find anything, then no more processing is necessary
                    // for this input.
                    continue;
                }
                Option::Some(matching_type_index) => {
                    // Otherwise, find the data type of the value given in the function
                    // call and add it as a possibility.
                    let possibility = self.program.get_entity_data_type(input.get_base());
                    match &real_types[matching_type_index] {
                        Option::None => real_types[matching_type_index] = Option::Some(possibility),
                        Option::Some(other_possibility) => {
                            real_types[matching_type_index] =
                                Option::Some(biggest_common_type(&possibility, other_possibility))
                        }
                    }
                }
            }
        }

        // Iterate over all the outputs to the function call.
        for (index, output) in func_call.iterate_over_outputs().enumerate() {
            let func_target_output = func_target.get_output(index);
            match get_type_index(self.program, func_target_output) {
                Option::None => {
                    // If we didn't find anything, then no more processing is necessary
                    // for this output.
                    continue;
                }
                Option::Some(matching_type_index) => {
                    // Otherwise, find the data type of the value given in the function
                    // call and add it as a possibility.
                    let possibility = self.program.get_entity_data_type(output.get_base());
                    match &real_types[matching_type_index] {
                        Option::None => real_types[matching_type_index] = Option::Some(possibility),
                        Option::Some(other_possibility) => {
                            real_types[matching_type_index] =
                                Option::Some(biggest_common_type(&possibility, other_possibility))
                        }
                    }
                }
            }
        }
        
        // TODO: Error if any data types could not be resolved, including for builtin funcs.

        // Create a copy of the function body and resolve it using the computed actual values for
        // each template parameter, then switch the function call to call the new body. We don't
        // do this for builtin functions though, since they don't technically have a body.
        if !is_builtin {
            self.push_table();
            let mut adopted_types = Vec::new();
            for index in 0..type_parameters.len() {
                let type_parameter = type_parameters[index];
                let real_type = real_types[index].as_ref().expect(
                    // Well not really, but it's a todo.
                    "We just checked that all types could be resolved."
                );
                let adopted_type = self.program.adopt_entity(Entity::DataType(real_type.clone()));
                adopted_types.push(adopted_type);
                self.add_conversion(type_parameter, adopted_type);
            }
            let func_scope = func_target.get_body();
            let parent = self.program.get_scope_parent(func_scope).expect(
                "All scopes that are bodies of functions have parent scopes."
            );
            let new_scope = self.initial_copy(func_scope, Option::Some(parent));
            self.resolve_body(func_scope, new_scope);
            let mut new_function = FunctionEntity::new(new_scope);
            for input in func_target.iterate_over_inputs() {
                new_function.add_input(self.convert(*input));
            }
            for output in func_target.iterate_over_outputs() {
                new_function.add_output(self.convert(*output));
            }
            for adopted_type in adopted_types.into_iter() {
                self.program.define_intermediate(new_scope, adopted_type);
            }
            let new_id = self.program.adopt_and_define_intermediate(parent, Entity::Function(new_function));
            self.pop_table();
            // TODO: Maybe generate a name for the new function?
            func_call.set_function(new_id);
        }

        // Resolve any inputs or outputs with automatic data types.
        for (index, input) in func_call.iterate_over_inputs().enumerate() {
            // If the input doesn't use an automatic data type, no resolving
            // is necessary.
            // TODO: Resolving of automatic data types when there is array
            // element or object member access. Currently, this code only
            // works on raw variables.
            let automatic_type = self.program.get_entity_data_type(input.get_base());
            if !automatic_type.is_automatic() {
                continue;
            }
            let func_target_input = func_target.get_input(index);
            let temp_holder;
            let target_type = match get_type_index(self.program, func_target_input) {
                Option::None => {
                    // It is not a template parameter, so just use the
                    // literal type of the input.
                    temp_holder = self.program.get_entity_data_type(func_target_input);
                    &temp_holder
                }
                Option::Some(matching_type_index) => {
                    // It is a template parameter, so find out what that
                    // template paramter resolved to.
                    real_types[matching_type_index]
                        .as_ref()
                        .expect("Unresolved template parameters should have been handled earlier.")
                }
            };
            let resolved_type = Self::resolve_automatic_type(&automatic_type, target_type);
            let resolved_type_entity = self
                .program
                .adopt_and_define_intermediate(target_scope, Entity::DataType(resolved_type));
            self.program
                .modify_entity(input.get_base(), make_var(resolved_type_entity));
        }
        for (index, output) in func_call.iterate_over_outputs().enumerate() {
            // If the output doesn't use an automatic data type, no resolving
            // is necessary.
            // TODO: Resolving of automatic data types when there is array
            // element or object member access. Currently, this code only
            // works on raw variables.
            let automatic_type = self.program.get_entity_data_type(output.get_base());
            if !automatic_type.is_automatic() {
                continue;
            }
            let func_target_output = func_target.get_output(index);
            let temp_holder;
            let target_type = match get_type_index(self.program, func_target_output) {
                Option::None => {
                    // It is not a template parameter, so just use the
                    // literal type of the output.
                    temp_holder = self.program.get_entity_data_type(func_target_output);
                    &temp_holder
                }
                Option::Some(matching_type_index) => {
                    // It is a template parameter, so find out what that
                    // template paramter resolved to.
                    real_types[matching_type_index]
                        .as_ref()
                        .expect("Unresolved template parameters should have been handled earlier.")
                }
            };
            let resolved_type = Self::resolve_automatic_type(&automatic_type, target_type);
            let resolved_type_entity = self
                .program
                .adopt_and_define_intermediate(target_scope, Entity::DataType(resolved_type));
            self.program
                .modify_entity(output.get_base(), make_var(resolved_type_entity));
        }
        self.program.add_func_call(target_scope, func_call);
    }

    fn resolve_body(&mut self, source: ScopeId, target: ScopeId) {
        for old_func_call in self.program.clone_scope_body(source).iter() {
            self.resolve_function_call(old_func_call, target);
        }

        if let Option::None = self.program.get_scope_parent(source) {
            if let Option::Some(func_id) = self.program.shallow_lookup_symbol(source, "main") {
                let function = match self.program.borrow_entity(func_id) {
                    Entity::Function(data) => data,
                    // TODO: Error if main is not a function.
                    _ => unreachable!(),
                };
                let body_id = function.get_body();
                drop(function);
                let resolved = self.resolve(body_id, Option::Some(target));
                // TODO: Copy inputs and outputs.
                let new_main = FunctionEntity::new(resolved);
                self.program
                    .adopt_and_define_symbol(target, "main", Entity::Function(new_main));
            }
        }
    }

    fn resolve(&mut self, source: ScopeId, parent: Option<ScopeId>) -> ScopeId {
        self.push_table();
        let copied = self.initial_copy(source, parent);
        self.resolve_body(source, copied);
        self.pop_table();
        copied
    }

    fn entry_point(&mut self, target: ScopeId) -> ScopeId {
        self.resolve(target, self.program.get_scope_parent(target))
    }
}
