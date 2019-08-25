use crate::vague::{
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
                        self.program.define_symbol(copy, &*name_value_pair.0, id);
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

    fn resolve_body(&mut self, source: ScopeId, target: ScopeId) {
        for old_func_call in self.program.clone_scope_body(source).iter() {
            let func_call = self.convert_func_call(&old_func_call);
            let func_target;
            // Get the FunctionEntity the function call is func_targeting.
            match self.program.borrow_entity(func_call.get_function()) {
                Entity::Variable(_data) => {
                    unimplemented!("Calling a function variable is unimplemented.")
                }
                Entity::Function(data) => func_target = data.clone(),
                Entity::BuiltinFunction(data) => func_target = data.get_base(),
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
                            Option::None => {
                                real_types[matching_type_index] = Option::Some(possibility)
                            }
                            Option::Some(other_possibility) => {
                                real_types[matching_type_index] = Option::Some(biggest_common_type(
                                    &possibility,
                                    other_possibility,
                                ))
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
                            Option::None => {
                                real_types[matching_type_index] = Option::Some(possibility)
                            }
                            Option::Some(other_possibility) => {
                                real_types[matching_type_index] = Option::Some(biggest_common_type(
                                    &possibility,
                                    other_possibility,
                                ))
                            }
                        }
                    }
                }
            }

            // TODO: Error if any data types could not be resolved.

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
                        real_types[matching_type_index].as_ref().expect(
                            "Unresolved template parameters should have been handled earlier.",
                        )
                    }
                };
                let resolved_type = Self::resolve_automatic_type(&automatic_type, target_type);
                let resolved_type_entity = self
                    .program
                    .adopt_and_define_intermediate(target, Entity::DataType(resolved_type));
                self.program
                    .redefine_entity(input.get_base(), make_var(resolved_type_entity));
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
                        real_types[matching_type_index].as_ref().expect(
                            "Unresolved template parameters should have been handled earlier.",
                        )
                    }
                };
                let resolved_type = Self::resolve_automatic_type(&automatic_type, target_type);
                let resolved_type_entity = self
                    .program
                    .adopt_and_define_intermediate(target, Entity::DataType(resolved_type));
                self.program
                    .redefine_entity(output.get_base(), make_var(resolved_type_entity));
            }
            self.program.add_func_call(target, func_call);
        } // End iterating over func calls.

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
