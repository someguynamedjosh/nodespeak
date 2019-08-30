use crate::structure;
use crate::structure::{
    DataType, FuncCall, FunctionData, KnownData, Program, ScopeId, VarAccess, Variable, VariableId,
};
use std::collections::HashMap;

pub fn resolve_scope(program: &mut Program, scope: ScopeId) -> ScopeId {
    let mut resolver = ScopeResolver::new(program);
    let result = resolver.entry_point(scope);
    result
}

struct ScopeResolver<'a> {
    program: &'a mut Program,
    conversion_table: HashMap<VariableId, VariableId>,
    // Even though VariableIds are global (so we don't have to worry about id
    // conflicts), we still have to worry about a single variable having
    // multiple conversions. For example, type parameters can be resolved to
    // different values depending on the types used for the inputs and outputs
    // of the function.
    conversion_stack: Vec<HashMap<VariableId, VariableId>>,
}

impl<'a> ScopeResolver<'a> {
    fn new(program: &'a mut Program) -> ScopeResolver<'a> {
        ScopeResolver {
            program,
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

    fn add_conversion(&mut self, from: VariableId, to: VariableId) {
        assert!(
            !self.conversion_table.contains_key(&from),
            "Cannot have multiple conversions for a single variable."
        );
        self.conversion_table.insert(from, to);
    }

    fn convert(&self, from: VariableId) -> VariableId {
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

    fn convert_function(&self, func_data: &FunctionData, new_body: ScopeId) -> FunctionData {
        assert!(!func_data.is_builtin());
        let mut result = FunctionData::new(new_body, func_data.get_header().clone());
        for old_input in func_data.borrow_inputs().iter() {
            result.add_input(self.convert(*old_input));
        }
        for old_output in func_data.borrow_outputs().iter() {
            result.add_output(self.convert(*old_output));
        }
        result
    }

    fn copy_scope(&mut self, source: ScopeId, parent: Option<ScopeId>) -> ScopeId {
        let copy = match parent {
            Option::Some(parent_id) => self.program.create_child_scope(parent_id),
            Option::None => self.program.create_scope(),
        };

        // TODO: We probably don't need to preserve the variable names in the
        // resolved scope. Depends on how some meta features get implemented in
        // the future.
        let symbol_table = self.program.clone_symbols_in(source);
        for name_value_pair in symbol_table.iter() {
            let old = *name_value_pair.1;
            let variable = self.program.borrow_variable(old).clone();
            // TODO: we might not need to clone every variable.
            let new = self
                .program
                .adopt_and_define_symbol(copy, name_value_pair.0, variable);
            self.add_conversion(old, new)
        }

        let intermediate_list = self.program.clone_intermediates_in(source);
        for old in intermediate_list.into_iter() {
            let variable = self.program.borrow_variable(old).clone();
            // TODO: we might not need to clone every variable.
            let new = self.program.adopt_and_define_intermediate(copy, variable);
            self.add_conversion(old, new)
        }

        copy
    }

    fn combine_type_into_table(
        template_parameter: VariableId,
        value: &DataType,
        table: &mut HashMap<VariableId, DataType>,
    ) {
        // If there is already a possible value for that template parameter...
        if table.contains_key(&template_parameter) {
            // Set the new value for that template parameter to the BCT between the old value and
            // the data type of the specific argument.
            table.insert(
                template_parameter,
                structure::biggest_common_type(&table[&template_parameter], value),
            );
        } else {
            table.insert(template_parameter, value.clone());
        }
    }

    fn resolve_dynamic_data_type(&mut self, data_type: DataType) -> DataType {
        match data_type {
            DataType::Dynamic(target) | DataType::LoadTemplateParameter(target) => {
                let target_variable = self.program.borrow_variable(target);
                if target_variable.is_permanent() {
                    if let KnownData::DataType(real_type) = target_variable.borrow_initial_value() {
                        let cloned = real_type.clone();
                        return self.resolve_dynamic_data_type(cloned);
                    }
                }
            }
            // Otherwise, the data type is already resolved.
            _ => (),
        }
        // If we did not convert, return the original type.
        data_type.clone()
    }

    fn resolve_dynamic_data_types(&mut self, from: ScopeId) {
        let symbol_table = self.program.clone_symbols_in(from);
        let intermediate_list = self.program.clone_intermediates_in(from);
        let var_iter = symbol_table
            .iter()
            .map(|pair| pair.1)
            .chain(intermediate_list.iter());
        for id in var_iter {
            if !self.program.is_permanent(*id) {
                // The only data types that aren't permanent are data type variables that will be
                // copied from other permanent data types, so we can afford to only process
                // permanent types.
                continue;
            }
            // Resolve the data type of the variable.
            let cloned_type = self.program.borrow_variable(*id).borrow_data_type().clone();
            let resolved_type = self.resolve_dynamic_data_type(cloned_type);
            self.program.set_data_type(*id, resolved_type);
            // If the variable is storing a data type, resolve that data type.
            if let KnownData::DataType(dtype) =
                self.program.borrow_variable(*id).borrow_initial_value()
            {
                let cloned = dtype.clone();
                let resolved_data = KnownData::DataType(self.resolve_dynamic_data_type(cloned));
                self.program.set_initial_value(*id, resolved_data);
            }
        }
    }

    fn resolve_template_parameters(
        parameter_type: &DataType,
        argument_type: &DataType,
        type_parameters: &mut HashMap<VariableId, DataType>,
        int_parameters: &mut HashMap<VariableId, i64>,
    ) {
        match parameter_type {
            // If the parameter type directly contributes to a template parameter...
            DataType::LoadTemplateParameter(type_var) => {
                // And if the argument type isn't automatic...
                Self::combine_type_into_table(*type_var, argument_type, type_parameters);
            }
            _ => (),
        }
    }

    fn resolve_automatic_type(&mut self, target: &VarAccess, real_type: DataType) {
        // For now, we don't have code to handle arrays.
        assert!(target.borrow_indexes().len() == 0);
        self.program.set_data_type(target.get_base(), real_type);
    }

    fn resolve_templated_type(
        data_type: &DataType,
        type_params: &HashMap<VariableId, DataType>,
        int_params: &HashMap<VariableId, i64>,
    ) -> DataType {
        match data_type {
            DataType::Dynamic(target) | DataType::LoadTemplateParameter(target) => type_params
                .get(target)
                .expect("Caller should have checked that all template parameters were resolved.")
                .clone(),
            _ => data_type.clone(),
        }
    }

    fn resolve_function_call(&mut self, old_func_call: &FuncCall, output: ScopeId) {
        let new_func_call = self.convert_func_call(old_func_call);
        let func_var = self.program.borrow_variable(new_func_call.get_function());
        let func_target;
        match func_var.borrow_initial_value() {
            KnownData::Function(data) => func_target = data.clone(),
            _ => panic!("TODO nice error for a function being vague."),
        }

        let mut type_params = HashMap::new();
        let mut int_params = HashMap::new();

        if func_target.borrow_inputs().len() != new_func_call.borrow_inputs().len() {
            panic!("TODO Nice error")
        }

        if func_target.borrow_outputs().len() != new_func_call.borrow_outputs().len() {
            panic!("TODO Nice error")
        }

        // Get iterators for the input and output parameters in the function definition.
        let input_params = func_target.borrow_inputs().iter();
        let output_params = func_target.borrow_outputs().iter();
        // Chain them together and then convert them to references to their data types. This makes
        // an iterator over the data types of each parameter.
        let param_types = input_params
            .chain(output_params)
            .map(|param_id| self.program.borrow_variable(*param_id).borrow_data_type());

        // Get iterators for the input and output arguments in the function call.
        let input_args = new_func_call.borrow_inputs().iter();
        let output_args = new_func_call.borrow_outputs().iter();
        // Chain them together and then convert them to references to their data types. This makes
        // an iterator over the data types of each argument.
        // TODO: Handle array types and such.
        let arg_types = input_args.chain(output_args).map(|arg_accessor| {
            self.program
                .borrow_variable(arg_accessor.get_base())
                .borrow_data_type()
        });

        // Since we already checked that the number of inputs and outputs are consistent across the
        // parameters and arguments, we can zip the parameter and argument iterators together and
        // know that each argument will correspond with the appropriate parameter.
        for (param_type, arg_type) in param_types.zip(arg_types) {
            // Figure out how to modify what we think the template parameters should be based on the
            // data type of the argument being used to set the parameter.
            Self::resolve_template_parameters(
                param_type,
                arg_type,
                &mut type_params,
                &mut int_params,
            );
        }

        // Create a copy of the function body and set all the function parameters inside it. Then
        // use that to resolve automatic variables and cast any arguments if needed. For builtin
        // functions, we use a different method because cloning their scopes all the time would be
        // very costly.
        if func_target.is_builtin() {
            let resolved_input_types: Vec<DataType> = func_target
                .borrow_inputs()
                .iter()
                .map(|input_id| {
                    Self::resolve_templated_type(
                        self.program.borrow_variable(*input_id).borrow_data_type(),
                        &type_params,
                        &int_params,
                    )
                })
                .collect();
            let resolved_output_types: Vec<DataType> = func_target
                .borrow_outputs()
                .iter()
                .map(|output_id| {
                    Self::resolve_templated_type(
                        self.program.borrow_variable(*output_id).borrow_data_type(),
                        &type_params,
                        &int_params,
                    )
                })
                .collect();
            let mut new_new_func_call = FuncCall::new(new_func_call.get_function());

            // Resolve the data type of any outputs that are automatic.
            for (index, output_accessor) in new_func_call.borrow_outputs().iter().enumerate() {
                if output_accessor
                    .borrow_data_type(self.program)
                    .is_automatic()
                {
                    self.resolve_automatic_type(
                        output_accessor,
                        resolved_output_types[index].clone(),
                    );
                }
            }

            // Cast any inputs to the function call.
            for (index, input_accessor) in new_func_call.borrow_inputs().iter().enumerate() {
                let input_type = &resolved_input_types[index];
                if input_accessor.borrow_data_type(self.program) == input_type {
                    // If the data types are identical, no casting is required.
                    new_new_func_call.add_input(input_accessor.clone());
                    continue;
                }
                let casted_var = Variable::variable(input_type.clone(), None);
                let casted_id = self
                    .program
                    .adopt_and_define_intermediate(output, casted_var);
                structure::create_cast(
                    self.program,
                    output,
                    input_accessor.clone(),
                    VarAccess::new(casted_id),
                );
                new_new_func_call.add_input(VarAccess::new(casted_id));
            }
            // Figure out what outputs will need to be casted. We can't cast them yet because
            // the actual function needs to be executed first before we will have outputs to cast.
            // We can't do this later because once we add the function call to the scope, we won't
            // be able to modify it anymore.
            let mut output_casts: Vec<(VarAccess, VarAccess)> = Vec::new();
            for (index, output_accessor) in new_func_call.borrow_outputs().iter().enumerate() {
                let output_type = &resolved_output_types[index];
                if output_accessor.borrow_data_type(self.program) == output_type {
                    new_new_func_call.add_output(output_accessor.clone());
                    continue;
                }
                let output_holder = Variable::variable(output_type.clone(), None);
                let holder_id = self
                    .program
                    .adopt_and_define_intermediate(output, output_holder);
                output_casts.push((VarAccess::new(holder_id), output_accessor.clone()));
                new_new_func_call.add_output(VarAccess::new(holder_id));
            }
            self.program.add_func_call(output, new_new_func_call);
            for (from, to) in output_casts.into_iter() {
                structure::create_cast(self.program, output, from, to);
            }
        } else {
            self.push_table();
            // Copy the function body.
            // TODO: Proper parent?
            let copied = self.copy_scope(func_target.get_body(), None);
            // Set all the type parameters in the copied body to have the values we determined they
            // should have earlier.
            for (type_parameter, resolved_value) in type_params.iter() {
                let resolved_parameter = self.convert(*type_parameter);
                self.program.set_initial_value(
                    resolved_parameter,
                    KnownData::DataType(resolved_value.clone()),
                );
            }
            // Get rid of any dynamic data types that were pointing to things we just resolved.
            self.resolve_dynamic_data_types(copied);
            // Resolve the function calls in the body.
            self.resolve_body(func_target.get_body(), copied);
            // Make a new function to hold the new function body. Our previous calls to copy the
            // scope and resolve its data types will also cause the data types of the parameters of
            // this new function to have resolved data types as well without requiring any extra
            // work.
            let new_function = self.convert_function(&func_target, copied);
            let new_function_id = self.program.adopt_and_define_intermediate(
                output,
                Variable::function_def(new_function.clone()),
            );
            let mut new_new_func_call = FuncCall::new(new_function_id);

            // Resolve the data type of any outputs that are automatic.
            for (index, output_id) in new_func_call.borrow_outputs().iter().enumerate() {
                if output_id.borrow_data_type(self.program).is_automatic() {
                    let target_var = self.program.borrow_variable(new_function.get_output(index));
                    let data_type = target_var.borrow_data_type().clone();
                    self.resolve_automatic_type(output_id, data_type);
                }
            }

            // Cast any inputs to the function call.
            for (index, input_accessor) in new_func_call.borrow_inputs().iter().enumerate() {
                let input_type = self
                    .program
                    .borrow_variable(new_function.get_input(index))
                    .borrow_data_type();
                if input_accessor.borrow_data_type(self.program) == input_type {
                    // If the data types are identical, no casting is required.
                    new_new_func_call.add_input(input_accessor.clone());
                    continue;
                }
                let casted_var = Variable::variable(input_type.clone(), None);
                let casted_id = self
                    .program
                    .adopt_and_define_intermediate(output, casted_var);
                structure::create_cast(
                    self.program,
                    output,
                    input_accessor.clone(),
                    VarAccess::new(casted_id),
                );
                new_new_func_call.add_input(VarAccess::new(casted_id));
            }
            // Figure out what outputs will need to be casted. We can't cast them yet because
            // the actual function needs to be executed first before we will have outputs to cast.
            // We can't do this later because once we add the function call to the scope, we won't
            // be able to modify it anymore.
            let mut output_casts: Vec<(VarAccess, VarAccess)> = Vec::new();
            for (index, output_accessor) in new_func_call.borrow_outputs().iter().enumerate() {
                let output_type = self
                    .program
                    .borrow_variable(new_function.get_output(index))
                    .borrow_data_type();
                if output_accessor.borrow_data_type(self.program) == output_type {
                    new_new_func_call.add_output(output_accessor.clone());
                    continue;
                }
                let output_holder = Variable::variable(output_type.clone(), None);
                let holder_id = self
                    .program
                    .adopt_and_define_intermediate(output, output_holder);
                output_casts.push((VarAccess::new(holder_id), output_accessor.clone()));
                new_new_func_call.add_output(VarAccess::new(holder_id));
            }
            self.program.add_func_call(output, new_new_func_call);
            for (from, to) in output_casts.into_iter() {
                structure::create_cast(self.program, output, from, to);
            }

            self.pop_table();
        }
    }

    fn resolve_body(&mut self, source: ScopeId, destination: ScopeId) {
        for old_func_call in self.program.clone_scope_body(source).into_iter() {
            self.resolve_function_call(&old_func_call, destination);
        }
    }

    fn entry_point(&mut self, target: ScopeId) -> ScopeId {
        let copy = self.copy_scope(target, self.program.get_scope_parent(target));
        self.resolve_body(target, copy);
        copy
    }
}
