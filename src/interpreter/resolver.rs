use crate::problem::{self, CompileProblem, FilePosition};
use crate::structure::{
    BaseType, BinaryOperator, DataType, Expression, FunctionData, KnownData, Program, ScopeId,
    VariableId,
};
use crate::util::NVec;
use std::borrow::Borrow;
use std::collections::HashMap;

pub fn resolve_scope(program: &mut Program, scope: ScopeId) -> Result<ScopeId, CompileProblem> {
    let mut resolver = ScopeResolver::new(program);
    resolver.entry_point(scope)
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

enum Content {
    /// A simpler or resolved version of the expression was found.
    Modified(Expression),
    /// The entire value of the expression has a determinate value.
    Interpreted(KnownData),
}

struct ResolvedExpression {
    pub content: Content,
    pub data_type: DataType,
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

    fn copy_scope(
        &mut self,
        source: ScopeId,
        parent: Option<ScopeId>,
    ) -> Result<ScopeId, CompileProblem> {
        let copy = match parent {
            Option::Some(parent_id) => self.program.create_child_scope(parent_id),
            Option::None => self.program.create_scope(),
        };

        // TODO: We probably don't need to preserve the variable names in the
        // resolved scope. Depends on how some meta features get implemented in
        // the future.
        let symbol_table = self.program[source].borrow_symbols().clone();
        for name_value_pair in symbol_table.iter() {
            let old = *name_value_pair.1;
            let variable = self.program[old].clone();
            // TODO: we might not need to clone every variable.
            let new = self
                .program
                .adopt_and_define_symbol(copy, name_value_pair.0, variable);
            self.add_conversion(old, new)
        }

        let intermediate_list = self.program[source].borrow_intermediates().clone();
        for old in intermediate_list.into_iter() {
            let variable = self.program[old].clone();
            // TODO: we might not need to clone every variable.
            let new = self.program.adopt_and_define_intermediate(copy, variable);
            self.add_conversion(old, new)
        }

        for old_input in self.program[source].borrow_inputs().clone() {
            let converted = self.convert(old_input);
            self.program[copy].add_input(converted);
        }

        for old_output in self.program[source].borrow_outputs().clone() {
            let converted = self.convert(old_output);
            self.program[copy].add_output(converted);
        }

        Result::Ok(copy)
    }

    fn resolve_access_expression(
        &mut self,
        resolved_base: ResolvedExpression,
        mut base_position: FilePosition,
        resolved_indexes: Vec<ResolvedExpression>,
        index_positions: Vec<FilePosition>,
        position: FilePosition,
    ) -> Result<ResolvedExpression, CompileProblem> {
        let final_type = resolved_base
            .data_type
            .clone_and_unwrap(resolved_indexes.len());
        Result::Ok(match resolved_base.content {
            // If the base of the access has a value known at compile time...
            Content::Interpreted(data) => {
                let mut base_data = match data {
                    KnownData::Array(data) => data,
                    _ => panic!("TODO: nice error, cannot index non-array."),
                };
                let mut known_indexes = Vec::new();
                let mut dynamic_indexes = Vec::new();
                let mut known = true;
                // Figure out how much of the indexing we can do at compile time.
                for (index, index_position) in resolved_indexes
                    .into_iter()
                    .zip(index_positions.into_iter())
                {
                    match index.content {
                        // If we know the value of the index...
                        Content::Interpreted(value) => {
                            // If we still know all the indexes up until this point...
                            if known {
                                base_position.include_other(&index_position);
                                // Store the integer value of the index.
                                known_indexes.push(match value {
                                    // TODO: check that number is positive.
                                    KnownData::Int(data) => data as usize,
                                    _ => panic!("TODO: nice error, index must be int."),
                                });
                            } else {
                                // Otherwise, add it as a dynamic (run-time) index.
                                dynamic_indexes.push(Expression::Literal(value, index_position));
                            }
                        }
                        // Otherwise, we will have to find the value at run time.
                        Content::Modified(expression) => {
                            // Once we hit an index that cannot be determined at compile time,
                            // everything after that must be used at run time.
                            known = false;
                            dynamic_indexes.push(expression);
                        }
                    }
                }
                // If we know at least one of the indexes right now at compile time...
                if known_indexes.len() > 0 {
                    // If the number of indexes we know result in a slice of the original array...
                    if known_indexes.len() < base_data.borrow_dimensions().len() {
                        assert!(
                            base_data.is_slice_inside(&known_indexes),
                            "TODO: nice error."
                        );
                        // Save only that slice of data to be used for our final output.
                        base_data = base_data.clone_slice(&known_indexes);
                    } else {
                        // Otherwise, we know what specific element is going to be accessed at
                        // compile time.
                        assert!(
                            known_indexes.len() == base_data.borrow_dimensions().len(),
                            "TODO: nice error."
                        );
                        // Check that the index is in the array.
                        assert!(base_data.is_inside(&known_indexes), "TODO: nice error");
                        // Check that there are no extra dynamic indexes. (They would be indexing
                        // a scalar type and thus be erroneous.)
                        assert!(dynamic_indexes.len() == 0, "TODO: nice error");
                        let element = base_data.borrow_item(&known_indexes).clone();
                        // Special case return of that specific element.
                        return Result::Ok(ResolvedExpression {
                            content: Content::Interpreted(element),
                            data_type: final_type,
                        });
                    }
                }

                // If there are no extra dynamic indexes, we can just return whatever portion of the
                // data that we were able to collect at compile time.
                if dynamic_indexes.len() == 0 {
                    ResolvedExpression {
                        content: Content::Interpreted(KnownData::Array(base_data)),
                        data_type: final_type,
                    }
                // Otherwise, make an access expression using the leftover dynamic indexes.
                } else {
                    ResolvedExpression {
                        content: Content::Modified(Expression::Access {
                            base: Box::new(Expression::Literal(
                                KnownData::Array(base_data),
                                base_position,
                            )),
                            indexes: dynamic_indexes,
                            position,
                        }),
                        data_type: final_type,
                    }
                }
            }
            // If the base of the access does not have a value known at compile time...
            Content::Modified(new_base) => {
                let mut new_indexes = Vec::with_capacity(resolved_indexes.len());
                for (index, index_position) in resolved_indexes
                    .into_iter()
                    .zip(index_positions.into_iter())
                {
                    if !index.data_type.is_specific_scalar(&BaseType::Int) {
                        return Result::Err(problem::array_index_not_int(
                            index_position,
                            &index.data_type,
                            base_position,
                        ));
                    }
                    match index.content {
                        // If we know the value of the index, make a literal expression out of it.
                        Content::Interpreted(value) => {
                            new_indexes.push(Expression::Literal(value, index_position));
                        }
                        // Otherwise, keep the simplified expression.
                        Content::Modified(expression) => {
                            new_indexes.push(expression);
                        }
                    }
                }
                // Make an access expression using the new indexes.
                ResolvedExpression {
                    content: Content::Modified(Expression::Access {
                        base: Box::new(new_base),
                        indexes: new_indexes,
                        position,
                    }),
                    data_type: final_type,
                }
            }
        })
    }

    fn resolve_binary_expression(
        &mut self,
        resolved_operand_1: ResolvedExpression,
        operand_1_position: FilePosition,
        operator: BinaryOperator,
        resolved_operand_2: ResolvedExpression,
        operand_2_position: FilePosition,
        position: FilePosition,
    ) -> Result<ResolvedExpression, CompileProblem> {
        let result_type = match super::util::biggest_type(
            &resolved_operand_1.data_type,
            &resolved_operand_2.data_type,
            self.program,
        ) {
            Result::Ok(rtype) => rtype,
            Result::Err(..) => {
                return Result::Err(problem::no_bct(
                    position,
                    operand_1_position,
                    &resolved_operand_1.data_type,
                    operand_2_position,
                    &resolved_operand_2.data_type,
                ))
            }
        };
        // TODO: Generate proxies when necessary.
        let content = match resolved_operand_1.content {
            Content::Interpreted(dat1) => match resolved_operand_2.content {
                // Both values were interpreted, so the value of the whole binary
                // expression can be computed.
                Content::Interpreted(dat2) => Content::Interpreted(
                    super::util::compute_binary_operation(&dat1, operator, &dat2),
                ),
                // LHS was interpreted, RHS could not be. Make LHS a literal and return
                // the resulting expression.
                Content::Modified(expr2) => Content::Modified(Expression::BinaryOperation(
                    Box::new(Expression::Literal(dat1, operand_1_position)),
                    operator,
                    Box::new(expr2),
                    position,
                )),
            },
            Content::Modified(expr1) => match resolved_operand_2.content {
                // RHS was interpreted, LHS could not be. Make RHS a literal and return
                // the resulting expression.
                Content::Interpreted(dat2) => Content::Modified(Expression::BinaryOperation(
                    Box::new(expr1),
                    operator,
                    Box::new(Expression::Literal(dat2, operand_2_position)),
                    position,
                )),
                // LHS and RHS couldn't be interpreted, only simplified.
                Content::Modified(expr2) => Content::Modified(Expression::BinaryOperation(
                    Box::new(expr1),
                    operator,
                    Box::new(expr2),
                    position,
                )),
            },
        };
        Result::Ok(ResolvedExpression {
            content,
            data_type: result_type,
        })
    }

    // Does not return an interpreted result, only a modified result. Since assignment *writes* to a
    // value instead of reads from it, there is no way to return an "interpreted" result.
    fn resolve_assignment_access_expression(
        &mut self,
        access_expression: &Expression,
    ) -> Result<Expression, CompileProblem> {
        Result::Ok(match access_expression {
            Expression::Variable(id, position) => {
                Expression::Variable(self.convert(*id), position.clone())
            }
            Expression::Access {
                base,
                indexes,
                position,
            } => {
                let resolved_base = self.resolve_assignment_access_expression(base)?;
                let mut resolved_indexes = Vec::with_capacity(indexes.len());
                for index in indexes {
                    let index_position = index.clone_position();
                    let resolved_index = self.resolve_expression(index)?;
                    resolved_indexes.push(match resolved_index.content {
                        Content::Interpreted(value) => Expression::Literal(value, index_position),
                        Content::Modified(expression) => expression,
                    });
                }
                Expression::Access {
                    base: Box::new(resolved_base),
                    indexes: resolved_indexes,
                    position: position.clone(),
                }
            }
            Expression::InlineReturn(..) => access_expression.clone(),
            _ => unreachable!("No other expression types found in assignment access expressions."),
        })
    }

    // Tries to assign known data to an access expression. The access expression must be
    // either a variable or an access expression. Indexes in the access expression can be
    // any kind of expression. If the value cannot be assigned at compile time, a modified
    // expression to assign the value at run time will be returned instead.
    fn assign_value_to_expression(
        &mut self,
        target: &Expression,
        value: KnownData,
        position: FilePosition,
    ) -> Result<Result<(), Expression>, CompileProblem> {
        Result::Ok(match target {
            Expression::Variable(id, ..) => {
                let converted = self.convert(*id);
                self.program[converted].set_temporary_value(value);
                Result::Ok(())
            }
            Expression::Access { base, indexes, .. } => {
                let (base_var_id, base_var_pos) = match base.borrow() {
                    Expression::Variable(id, position) => (self.convert(*id), position.clone()),
                    _ => unreachable!("Nothing else can be the base of an access."),
                };
                let mut all_indexes_known = true;
                let mut known_indexes = Vec::new();
                let mut resolved_indexes = Vec::with_capacity(indexes.len());
                for index in indexes {
                    let index_position = index.clone_position();
                    let result = self.resolve_expression(index)?;
                    match result.content {
                        Content::Interpreted(value) => {
                            if all_indexes_known {
                                if let KnownData::Int(value) = value {
                                    // TODO: Check that value is positive.
                                    known_indexes.push(value as usize);
                                } else {
                                    panic!("TODO: nice error, index must be int.");
                                }
                            }
                            // Also put a literal into resolved indexes in case we don't know all
                            // the indexes and need to construct a runtime expression to set the
                            // value.
                            resolved_indexes.push(Expression::Literal(value, index_position));
                        }
                        Content::Modified(expression) => {
                            resolved_indexes.push(expression);
                            all_indexes_known = false;
                        }
                    }
                }
                if all_indexes_known {
                    self.program[base_var_id]
                        .borrow_temporary_value_mut()
                        .require_array_mut()
                        .set_item(&known_indexes, value);
                    Result::Ok(())
                } else {
                    Result::Err(Expression::Access {
                        base: Box::new(Expression::Variable(base_var_id, base_var_pos)),
                        indexes: resolved_indexes,
                        position,
                    })
                }
            }
            _ => unreachable!("Assignment target cannot contain any other expressions."),
        })
    }

    fn assign_unknown_to_expression(&mut self, target: &Expression) {
        match target {
            Expression::Variable(id, ..) => {
                let converted = self.convert(*id);
                self.program[converted].set_temporary_value(KnownData::Unknown);
            }
            Expression::Access { base, .. } => {
                // TODO: Something more efficient that takes into account any known indexes.
                let (base_var_id, _base_var_pos) = match base.borrow() {
                    Expression::Variable(id, position) => (self.convert(*id), position.clone()),
                    _ => unreachable!("Nothing else can be the base of an access."),
                };
                self.program[base_var_id].set_temporary_value(KnownData::Unknown);
            }
            _ => unreachable!("Cannot have anything else in an assignment expression."),
        }
    }

    fn resolve_func_call(
        &mut self,
        function: &Expression,
        inputs: &Vec<Expression>,
        outputs: &Vec<Expression>,
        position: &FilePosition,
    ) -> Result<ResolvedExpression, CompileProblem> {
        // We want to make a new table specifically for this function, so that any variable
        // conversions we make won't be applied to other calls to the same funciton.
        self.push_table();
        // Make sure we can figure out what function is being called right now at compile time.
        let resolved_function = match self.resolve_expression(function.borrow())?.content {
            Content::Interpreted(value) => value,
            Content::Modified(..) => {
                return Result::Err(problem::vague_function(
                    position.clone(),
                    function.clone_position(),
                ))
            }
        };
        // Get the actual function data.
        let function_data = match resolved_function {
            KnownData::Function(data) => data,
            _ => return Result::Err(problem::not_function(function.clone_position())),
        };
        // Make a copy of the function body.
        let old_function_body = function_data.get_body();
        let new_function_body = self.copy_scope(
            old_function_body,
            self.program[old_function_body].get_parent(),
        )?;
        // Reset the temporary values of all the inputs and outputs of the new body.
        // TODO: Not sure if this step is necessary.
        let input_targets = self.program[new_function_body].borrow_inputs().clone();
        let output_targets = self.program[new_function_body].borrow_outputs().clone();
        for target in input_targets.iter().chain(output_targets.iter()) {
            self.program[*target].reset_temporary_value();
        }

        // Try to set the value of any input parameters at compile time when the corresponding
        // argument has a value that can be determined at compile time.
        let mut runtime_inputs = Vec::new();
        for (expression, target) in inputs.iter().zip(input_targets.iter()) {
            match self.resolve_expression(expression)?.content {
                // If we know the value of the argument, use that to set the parameter.
                Content::Interpreted(value) => self.program[*target].set_temporary_value(value),
                // Otherwise, store the simplified expression for the argument as well as the
                // parameter it corresponds to for later use.
                Content::Modified(new_expression) => runtime_inputs.push((new_expression, *target)),
            }
        }

        // Now that we have given specific values to as many inputs as possible, resolve each
        // expression in the function body. Hopefully it will end up eliminating some of the
        // instructions, resulting in a simpler body.
        for expression in self.program[old_function_body].borrow_body().clone() {
            match self.resolve_expression(&expression)?.content {
                // TODO: Warn if an output is not being used.
                Content::Interpreted(..) => (),
                Content::Modified(new_expression) => match new_expression {
                    Expression::Return(..) => break,
                    _ => self.program[old_function_body].add_expression(new_expression),
                },
            }
        }
        // Because making a scope copy using the self::copy function adds a bunch of conversions to
        // the internal conversion table, and because that table is used when expressions are
        // resolved, then we can read the values of the output variables on the new body to see if
        // any of them have compile-time determinable values.
        // TODO: This logic only works if the function has only determinate return statements! (I.E.
        // no return statements inside branches or loops.)
        // This variable holds the value of the inline return. This algorithm is currently designed
        // to work only under the condition that if a function call has an inline return value, then
        // there are no other return values. This requirement is currently enforced by the grammar,
        // which requires that all function calls used inside of an expression (and thus having an
        // implicit inline return) must have one and only one return value.
        // TODO: This also implicitly assumes that functions do not have side effects. Need to check
        // that a function does not cause any side effects.
        let mut inline_result = Option::None;
        let mut runtime_outputs = Vec::new();
        let mut runtime_inline_output_type = Option::None;
        for (target_access, source) in outputs.iter().zip(output_targets.iter()) {
            let source_data = self.program[*source].borrow_temporary_value();
            // If we don't know what the output will be, we need to leave it in the code to be
            // computed at runtime.
            if let KnownData::Unknown = source_data {
                runtime_outputs.push((target_access.clone(), *source));
                if let Expression::InlineReturn(..) = target_access {
                    runtime_inline_output_type =
                        Option::Some(self.program[*source].borrow_data_type().clone());
                }
            } else {
                let cloned_data = source_data.clone();
                // If the output 'target' is InlineReturn, then we have no variable we can assign
                // the data to. Instead, return it as the interpreted result.
                if let Expression::InlineReturn(..) = target_access {
                    // TODO: Check that there is only one inline return.
                    inline_result = Option::Some(ResolvedExpression {
                        content: Content::Interpreted(cloned_data),
                        data_type: self.program[*source].borrow_data_type().clone(),
                    });
                    continue;
                }
                // Check to see if we can write the known value of the output parameter to the
                // output argument.
                match self.assign_value_to_expression(
                    target_access,
                    cloned_data.clone(),
                    FilePosition::placeholder(),
                )? {
                    // Don't need to do anything, we successfully set the value of the argument.
                    Result::Ok(..) => (),
                    // We know the value the output parameter will have, but we have to assign
                    // it at runtime. Tack on some code to assign the value.
                    Result::Err(..) => {
                        // TODO?: File positions, though I'm not sure if there's any sensible
                        // option in this case.
                        self.program[new_function_body].add_expression(Expression::Assign {
                            target: Box::new(Expression::Variable(
                                *source,
                                FilePosition::placeholder(),
                            )),
                            value: Box::new(Expression::Literal(
                                cloned_data,
                                FilePosition::placeholder(),
                            )),
                            position: FilePosition::placeholder(),
                        });
                        runtime_outputs.push((target_access.clone(), *source));
                    }
                }
            }
        }

        // Clear the input and output list in the new function body, then add back only the inputs
        // and outputs which are required at run time.
        let scope = &mut self.program[new_function_body];
        scope.clear_inputs();
        scope.clear_outputs();
        for (_, parameter) in runtime_inputs.iter() {
            scope.add_input(*parameter);
        }
        for (_, parameter) in runtime_outputs.iter() {
            scope.add_output(*parameter);
        }

        // Make a literal containing the data of the new function we created.
        let new_function_data =
            FunctionData::new(new_function_body, function_data.get_header().clone());
        let new_function = Expression::Literal(
            KnownData::Function(new_function_data),
            function.clone_position(),
        );
        self.pop_table();

        // Make a function call expression using only the arguments for the runtime parameters.
        Result::Ok(match inline_result {
            Option::Some(result) => result,
            Option::None => {
                if runtime_outputs.len() == 0 {
                    ResolvedExpression {
                        content: Content::Interpreted(KnownData::Void),
                        data_type: DataType::scalar(BaseType::Void),
                    }
                } else {
                    let content = Content::Modified(Expression::FuncCall {
                        function: Box::new(new_function),
                        inputs: runtime_inputs.into_iter().map(|item| item.0).collect(),
                        outputs: runtime_outputs.into_iter().map(|item| item.0).collect(),
                        position: position.clone(),
                    });
                    let data_type =
                        runtime_inline_output_type.unwrap_or(DataType::scalar(BaseType::Void));
                    ResolvedExpression { content, data_type }
                }
            }
        })
    }

    fn resolve_expression(
        &mut self,
        old_expression: &Expression,
    ) -> Result<ResolvedExpression, CompileProblem> {
        Result::Ok(match old_expression {
            Expression::Literal(value, ..) => ResolvedExpression {
                data_type: value
                    .get_data_type()
                    .expect("Literals always have a known data type."),
                content: Content::Interpreted(value.clone()),
            },
            Expression::Variable(id, position) => {
                let converted_id = self.convert(*id);
                let temporary_value = self.program[converted_id].borrow_temporary_value();
                match temporary_value {
                    KnownData::Unknown => {
                        let content =
                            Content::Modified(Expression::Variable(converted_id, position.clone()));
                        let data_type = self.program[converted_id].borrow_data_type().clone();
                        ResolvedExpression { content, data_type }
                    }
                    _ => {
                        let content = Content::Interpreted(temporary_value.clone());
                        let data_type = temporary_value
                            .get_data_type()
                            .expect("Already checked that data was not uknown.");
                        ResolvedExpression { content, data_type }
                    }
                }
            }
            Expression::Proxy { .. } => unimplemented!(),
            Expression::Access {
                base,
                indexes,
                position,
            } => {
                // TODO: Optimize accessing values in big arrays so that we don't have to clone
                // the entire array to pick out one value.
                let base_position = base.clone_position();
                let resolved_base = self.resolve_expression(base)?;
                let mut index_positions = Vec::with_capacity(indexes.len());
                let mut resolved_indexes = Vec::with_capacity(indexes.len());
                for index in indexes {
                    index_positions.push(index.clone_position());
                    resolved_indexes.push(self.resolve_expression(index)?);
                }
                self.resolve_access_expression(
                    resolved_base,
                    base_position,
                    resolved_indexes,
                    index_positions,
                    position.clone(),
                )?
            }
            Expression::InlineReturn(..) => unimplemented!(),

            Expression::UnaryOperation(..) => unimplemented!(),
            Expression::BinaryOperation(operand_1, operator, operand_2, position) => {
                let operand_1_position = operand_1.clone_position();
                let resolved_operand_1 = self.resolve_expression(operand_1)?;
                let operand_2_position = operand_2.clone_position();
                let resolved_operand_2 = self.resolve_expression(operand_2)?;
                self.resolve_binary_expression(
                    resolved_operand_1,
                    operand_1_position,
                    *operator,
                    resolved_operand_2,
                    operand_2_position,
                    position.clone(),
                )?
            }

            Expression::Collect(values, position) => {
                let mut resolved_values = Vec::with_capacity(values.len());
                let mut value_positions = Vec::with_capacity(values.len());
                let mut all_known = true;
                for value in values {
                    value_positions.push(value.clone_position());
                    let resolved_value = self.resolve_expression(value)?;
                    if let Content::Modified(..) = &resolved_value.content {
                        all_known = false;
                    }
                    resolved_values.push(resolved_value);
                }
                if all_known {
                    let mut data = Vec::with_capacity(resolved_values.len());
                    for value in resolved_values {
                        data.push(match value.content {
                            Content::Interpreted(data) => data,
                            _ => unreachable!(
                                "We previously checked that all the data was interpreted."
                            ),
                        });
                    }
                    // TODO: Check that data types are compatible with each other.
                    if let KnownData::Array(..) = &data[0] {
                        // Get all the sub arrays and collect them into a higher-dimension array.
                        let mut sub_arrays = Vec::with_capacity(data.len());
                        for datum in data {
                            sub_arrays.push(match datum {
                                KnownData::Array(sub_array) => sub_array,
                                _ => panic!("TODO: nice error, data types incompatible."),
                            })
                        }
                        let final_data = KnownData::Array(NVec::collect(sub_arrays));
                        ResolvedExpression {
                            data_type: final_data
                                .get_data_type()
                                .expect("Data cannot be unknown, we just built it."),
                            content: Content::Interpreted(final_data),
                        }
                    } else {
                        // Each element is a scalar, so just make a new 1d array out of them.
                        let final_data = KnownData::Array(NVec::from_vec(data));
                        ResolvedExpression {
                            data_type: final_data
                                .get_data_type()
                                .expect("Data cannot be unknown, we just built it."),
                            content: Content::Interpreted(final_data),
                        }
                    }
                } else {
                    let mut items = Vec::with_capacity(resolved_values.len());
                    // TODO: Error for zero-sized arrays.
                    let data_type = resolved_values[0].data_type.clone();
                    // Standard treatment, fully interpreted values become literal expressions.
                    for (value, value_position) in
                        resolved_values.into_iter().zip(value_positions.into_iter())
                    {
                        // TODO: Check that all the elements are the same type.
                        match value.content {
                            Content::Interpreted(value) => {
                                items.push(Expression::Literal(value, value_position))
                            }
                            Content::Modified(expression) => items.push(expression),
                        }
                    }
                    ResolvedExpression {
                        content: Content::Modified(Expression::Collect(items, position.clone())),
                        data_type,
                    }
                }
            }
            Expression::CreationPoint(old_var_id, position) => {
                let new_var_id = self.convert(*old_var_id);
                let data_type = self.program[new_var_id].borrow_data_type();
                let mut new_sizes = Vec::with_capacity(data_type.borrow_dimensions().len());
                for size in data_type.borrow_dimensions().clone() {
                    match self.resolve_expression(&size)?.content {
                        Content::Interpreted(result) => {
                            if let KnownData::Int(value) = result {
                                new_sizes.push(value);
                            } else {
                                return Result::Err(problem::array_size_not_int(
                                    size.clone_position(),
                                    &result,
                                    position.clone(),
                                ));
                            }
                        }
                        _ => {
                            return Result::Err(problem::vague_array_size(
                                size.clone_position(),
                                position.clone(),
                            ))
                        }
                    }
                }
                self.program[new_var_id]
                    .borrow_data_type_mut()
                    .set_literal_dimensions(new_sizes);
                ResolvedExpression {
                    content: Content::Interpreted(KnownData::Void),
                    data_type: DataType::scalar(BaseType::Void),
                }
            }

            Expression::Assert(value, position) => {
                let resolved_value = self.resolve_expression(value)?;
                match resolved_value.content {
                    Content::Interpreted(value) => {
                        if let KnownData::Bool(succeed) = value {
                            if succeed {
                                ResolvedExpression {
                                    content: Content::Interpreted(KnownData::Void),
                                    data_type: DataType::scalar(BaseType::Void),
                                }
                            } else {
                                panic!("TODO: nice error, assert guaranteed to fail.")
                            }
                        } else {
                            panic!("TODO: nice error, argument to assert is not bool.")
                        }
                    }
                    Content::Modified(expression) => ResolvedExpression {
                        content: Content::Modified(Expression::Assert(
                            Box::new(expression),
                            position.clone(),
                        )),
                        data_type: DataType::scalar(BaseType::Void),
                    },
                }
            }
            Expression::Assign {
                target,
                value,
                position,
            } => match self.resolve_expression(value)?.content {
                Content::Interpreted(known_value) => {
                    let result =
                        self.assign_value_to_expression(target, known_value, position.clone())?;
                    if let Result::Err(resolved_expresion) = result {
                        ResolvedExpression {
                            content: Content::Modified(resolved_expresion),
                            data_type: DataType::scalar(BaseType::Void),
                        }
                    } else {
                        ResolvedExpression {
                            content: Content::Interpreted(KnownData::Void),
                            data_type: DataType::scalar(BaseType::Void),
                        }
                    }
                }
                Content::Modified(resolved_value) => {
                    let resolved_target = self.resolve_assignment_access_expression(target)?;
                    self.assign_unknown_to_expression(&resolved_target);
                    let content = Content::Modified(Expression::Assign {
                        target: Box::new(resolved_target),
                        value: Box::new(resolved_value),
                        position: position.clone(),
                    });
                    ResolvedExpression {
                        content,
                        data_type: DataType::scalar(BaseType::Void),
                    }
                }
            },
            Expression::Return(position) => {
                let content = Content::Modified(Expression::Return(position.clone()));
                ResolvedExpression {
                    content,
                    data_type: DataType::scalar(BaseType::Void),
                }
            }

            Expression::FuncCall {
                function,
                inputs,
                outputs,
                position,
            } => self.resolve_func_call(function, inputs, outputs, position)?,
        })
    }

    fn resolve_scope(&mut self, source: ScopeId) -> Result<ScopeId, CompileProblem> {
        let copied = self.copy_scope(source, self.program[source].get_parent())?;
        let old_body = self.program[source].borrow_body().clone();
        for expression in old_body {
            let resolved = self.resolve_expression(&expression)?;
            match resolved.content {
                // If it was successfully interpreted, we don't need to do anything.
                Content::Interpreted(..) => (),
                Content::Modified(expression) => match expression {
                    // TODO: Warn for expressions that have outputs that are not stored.
                    Expression::Return(..) => break,
                    _ => self.program[copied].add_expression(expression),
                },
            }
        }
        Result::Ok(copied)
    }

    fn entry_point(&mut self, entry_scope: ScopeId) -> Result<ScopeId, CompileProblem> {
        self.resolve_scope(entry_scope)
    }
}
