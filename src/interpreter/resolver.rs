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

    fn int_literal(value: i64, position: FilePosition) -> Expression {
        Expression::Literal(KnownData::Int(value), position)
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
        resolved_indexes: Vec<(ResolvedExpression, FilePosition)>,
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
                for (index, index_position) in resolved_indexes.into_iter() {
                    if !index.data_type.is_specific_scalar(&BaseType::Int) {
                        panic!(
                            "TODO: nice error, data type of index must be Int not {:?}",
                            index.data_type
                        )
                    }
                    match index.content {
                        // If we know the value of the index...
                        Content::Interpreted(KnownData::Int(value)) => {
                            // If we still know all the indexes up until this point...
                            if known {
                                base_position.include_other(&index_position);
                                // Store the integer value of the index.
                                known_indexes.push(value as usize);
                            } else {
                                // Otherwise, add it as a dynamic (run-time) index.
                                dynamic_indexes.push(Self::int_literal(value, index_position));
                            }
                        }
                        // If we know that the value isn't an int...
                        Content::Interpreted(_non_int_value) => {
                            unreachable!("Non-int value handled above.");
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
                for (index, index_position) in resolved_indexes.into_iter() {
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

    fn require_resolved_function_data(
        &self,
        from: VariableId,
    ) -> Result<FunctionData, CompileProblem> {
        match self.program[from].borrow_temporary_value() {
            KnownData::Unknown => panic!("TODO: nice error, vague function variable."),
            KnownData::Function(data) => Result::Ok(data.clone()),
            _ => panic!("TODO: variable does not contain a function."),
        }
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

    // Does not return an interpreted result, only a modified result. Since assignment *writes* to a
    // value instead of reads from it, there is no way to return an "interpreted" result.
    fn resolve_assignment_access_expression(
        &mut self,
        access_expression: &Expression,
    ) -> Result<(Expression, DataType), CompileProblem> {
        Result::Ok(match access_expression {
            Expression::Variable(id, position) => (
                Expression::Variable(self.convert(*id), position.clone()),
                self.program[self.convert(*id)].borrow_data_type().clone(),
            ),
            Expression::PickInput(function, index, position) => {
                let scope = self.require_resolved_function_data(*function)?.get_body();
                let converted = self.convert(self.program[scope].get_input(*index));
                (
                    Expression::Variable(converted, position.clone()),
                    self.program[converted].borrow_data_type().clone(),
                )
            }
            Expression::PickOutput(function, index, position) => {
                let scope = self.require_resolved_function_data(*function)?.get_body();
                let converted = self.convert(self.program[scope].get_output(*index));
                (
                    Expression::Variable(converted, position.clone()),
                    self.program[converted].borrow_data_type().clone(),
                )
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
                let num_indexes = resolved_indexes.len();
                let expr = Expression::Access {
                    base: Box::new(resolved_base.0),
                    indexes: resolved_indexes,
                    position: position.clone(),
                };
                (expr, resolved_base.1.clone_and_unwrap(num_indexes))
            }
            Expression::InlineReturn(..) => (
                access_expression.clone(),
                DataType::scalar(BaseType::Automatic),
            ),
            _ => unreachable!("No other expression types found in assignment access expressions."),
        })
    }

    fn resolve_assign_statement(
        &mut self,
        target: &Expression,
        value: &Expression,
        position: &FilePosition,
    ) -> Result<ResolvedExpression, CompileProblem> {
        let resolved_target = self.resolve_assignment_access_expression(target)?;
        let resolved_value = self.resolve_expression(value)?;
        if !resolved_value
            .data_type
            .equivalent(&resolved_target.1, self.program)
        {
            panic!("TODO: nice error, mismatched data types in assignment.");
        }
        Result::Ok(match resolved_value.content {
            Content::Interpreted(known_value) => {
                let result = self.assign_value_to_expression(
                    &resolved_target.0,
                    known_value,
                    position.clone(),
                )?;
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
                self.assign_unknown_to_expression(&resolved_target.0);
                let content = Content::Modified(Expression::Assign {
                    target: Box::new(resolved_target.0),
                    value: Box::new(resolved_value),
                    position: position.clone(),
                });
                ResolvedExpression {
                    content,
                    data_type: DataType::scalar(BaseType::Void),
                }
            }
        })
    }

    fn resolve_func_call(
        &mut self,
        function: &Expression,
        setup: &Vec<Expression>,
        teardown: &Vec<Expression>,
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

        // Try and do as much setup at compile time as possible.
        let mut runtime_setup = Vec::new();
        for expr in setup {
            match self.resolve_expression(expr)?.content {
                // Fully computed at compile time.
                Content::Interpreted(..) => (),
                // Still requires at least some computation during run time.
                Content::Modified(new_expression) => runtime_setup.push(new_expression),
            }
        }

        let mut empty_body = true;
        // Now try and simplify the body as much as possible.
        // TODO: This logic only works if the function has only determinate return statements! (I.E.
        // no return statements inside branches or loops.)
        // TODO: This also implicitly assumes that functions do not have side effects. Need to check
        // that a function does not cause any side effects.
        for expression in self.program[old_function_body].borrow_body().clone() {
            match self.resolve_expression(&expression)?.content {
                // TODO: Warn if an output is not being used.
                // Fully computed at compile time.
                Content::Interpreted(..) => (),
                // We still need to do something at run time.
                Content::Modified(new_expression) => match new_expression {
                    // If a return is unconditionally encountered, we can just skip the rest of the
                    // code in the scope.
                    Expression::Return(..) => break,
                    _ => {
                        empty_body = false;
                        self.program[new_function_body].add_expression(new_expression);
                    }
                },
            }
        }

        // Now try and resolve as much of the teardown as possible. Keep track of whether or not
        // there is an inline return statement and what value it contains.
        let mut runtime_teardown = Vec::new();
        let mut inline_output = None;
        for expr in teardown {
            match expr {
                Expression::InlineReturn(return_value, position) => {
                    let resolved = self.resolve_expression(return_value)?;
                    if let Content::Modified(new_expression) = &resolved.content {
                        runtime_teardown.push(Expression::InlineReturn(
                            Box::new(new_expression.clone()),
                            position.clone(),
                        ))
                    }
                    inline_output = Some(resolved);
                }
                _ => {
                    if let Content::Modified(new_expr) = self.resolve_expression(expr)?.content {
                        runtime_teardown.push(new_expr);
                    }
                }
            }
        }

        self.pop_table();

        Result::Ok(if empty_body && runtime_teardown.len() == 0 {
            match inline_output {
                Some(value) => value,
                None => ResolvedExpression {
                    content: Content::Interpreted(KnownData::Void),
                    data_type: DataType::scalar(BaseType::Void),
                },
            }
        } else {
            ResolvedExpression {
                content: Content::Modified(Expression::FuncCall {
                    function: Box::new(Expression::Literal(
                        KnownData::Function(FunctionData::new(
                            new_function_body,
                            function_data.get_header().clone(),
                        )),
                        function.clone_position(),
                    )),
                    setup: runtime_setup,
                    teardown: runtime_teardown,
                    position: position.clone(),
                }),
                data_type: match inline_output {
                    Some(resolved) => resolved.data_type,
                    None => DataType::scalar(BaseType::Void),
                },
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
                let mut resolved_indexes = Vec::with_capacity(indexes.len());
                for index in indexes {
                    resolved_indexes
                        .push((self.resolve_expression(index)?, index.clone_position()));
                }
                self.resolve_access_expression(
                    resolved_base,
                    base_position,
                    resolved_indexes,
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

            Expression::PickInput(..) => unreachable!("Should be handled elsewhere."),
            Expression::PickOutput(function, index, position) => {
                let scope = self.require_resolved_function_data(*function)?.get_body();
                let converted = self.convert(self.program[scope].get_output(*index));
                return self.resolve_expression(&Expression::Variable(converted, position.clone()));
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
            } => self.resolve_assign_statement(target, value, position)?,
            Expression::Return(position) => {
                let content = Content::Modified(Expression::Return(position.clone()));
                ResolvedExpression {
                    content,
                    data_type: DataType::scalar(BaseType::Void),
                }
            }

            Expression::FuncCall {
                function,
                setup,
                teardown,
                position,
            } => self.resolve_func_call(function, setup, teardown, position)?,
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
