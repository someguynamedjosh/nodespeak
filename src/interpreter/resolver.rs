use crate::problem::CompileProblem;
use crate::structure::{BinaryOperator, Expression, KnownData, Program, ScopeId, VariableId};
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

enum ResolveExpressionResult {
    Modified(Expression), // A simpler or resolved version of the expression was found.
    Interpreted(KnownData), // The entire value of the expression has a determinate value.
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

        for old_input in self.program.borrow_scope(source).borrow_inputs().clone() {
            let converted = self.convert(old_input);
            self.program.borrow_scope_mut(copy).add_input(converted);
        }

        for old_output in self.program.borrow_scope(source).borrow_outputs().clone() {
            let converted = self.convert(old_output);
            self.program.borrow_scope_mut(copy).add_output(converted);
        }

        Result::Ok(copy)
    }

    fn resolve_access_expression(
        &mut self,
        resolved_base: ResolveExpressionResult,
        resolved_indexes: Vec<ResolveExpressionResult>,
    ) -> Result<ResolveExpressionResult, CompileProblem> {
        Result::Ok(match resolved_base {
            // If the base of the access has a value known at compile time...
            ResolveExpressionResult::Interpreted(data) => {
                let mut base_data = match data {
                    KnownData::Array(data) => data,
                    _ => panic!("TODO: nice error, cannot index non-array."),
                };
                // Figure out how much of the indexing we can do at compile time.
                let mut known_indexes = Vec::new();
                let mut dynamic_indexes = Vec::new();
                let mut known = true;
                for index in resolved_indexes {
                    // Once we hit an index that cannot be determined at compile time, everything
                    // after that must be used at run time.
                    if let ResolveExpressionResult::Modified(..) = index {
                        known = false;
                    }
                    match index {
                        ResolveExpressionResult::Interpreted(value) => {
                            if known {
                                known_indexes.push(match value {
                                    // TODO: check that number is positive.
                                    KnownData::Int(data) => data as usize,
                                    _ => panic!("TODO: nice error, index must be int."),
                                });
                            } else {
                                dynamic_indexes.push(Expression::Literal(value));
                            }
                        }
                        ResolveExpressionResult::Modified(expression) => {
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
                        return Result::Ok(ResolveExpressionResult::Interpreted(element));
                    }
                }

                // If there are no extra dynamic indexes, we can just return whatever portion of the
                // data that we were able to collect at compile time.
                if dynamic_indexes.len() == 0 {
                    ResolveExpressionResult::Interpreted(KnownData::Array(base_data))
                // Otherwise, make an access expression using the leftover dynamic indexes.
                } else {
                    ResolveExpressionResult::Modified(Expression::Access {
                        base: Box::new(Expression::Literal(KnownData::Array(base_data))),
                        indexes: dynamic_indexes,
                    })
                }
            }
            // If the base of the access does not have a value known at compile time...
            ResolveExpressionResult::Modified(new_base) => {
                let mut new_indexes = Vec::with_capacity(resolved_indexes.len());
                for index in resolved_indexes.into_iter() {
                    match index {
                        // If we know the value of the index, make a literal expression out of it.
                        ResolveExpressionResult::Interpreted(value) => {
                            new_indexes.push(Expression::Literal(value));
                        }
                        // Otherwise, keep the simplified expression.
                        ResolveExpressionResult::Modified(expression) => {
                            new_indexes.push(expression);
                        }
                    }
                }
                // Make an access expression using the new indexes.
                ResolveExpressionResult::Modified(Expression::Access {
                    base: Box::new(new_base),
                    indexes: new_indexes,
                })
            }
        })
    }

    fn resolve_binary_expression(
        &mut self,
        resolved_operand_1: ResolveExpressionResult,
        operator: BinaryOperator,
        resolved_operand_2: ResolveExpressionResult,
    ) -> Result<ResolveExpressionResult, CompileProblem> {
        Result::Ok(match resolved_operand_1 {
            ResolveExpressionResult::Interpreted(dat1) => match resolved_operand_2 {
                // Both values were interpreted, so the value of the whole binary
                // expression can be computed.
                ResolveExpressionResult::Interpreted(dat2) => ResolveExpressionResult::Interpreted(
                    super::util::compute_binary_operation(&dat1, operator, &dat2),
                ),
                // LHS was interpreted, RHS could not be. Make LHS a literal and return
                // the resulting expression.
                ResolveExpressionResult::Modified(expr2) => {
                    ResolveExpressionResult::Modified(Expression::BinaryOperation(
                        Box::new(Expression::Literal(dat1)),
                        operator,
                        Box::new(expr2),
                    ))
                }
            },
            ResolveExpressionResult::Modified(expr1) => match resolved_operand_2 {
                // RHS was interpreted, LHS could not be. Make RHS a literal and return
                // the resulting expression.
                ResolveExpressionResult::Interpreted(dat2) => {
                    ResolveExpressionResult::Modified(Expression::BinaryOperation(
                        Box::new(expr1),
                        operator,
                        Box::new(Expression::Literal(dat2)),
                    ))
                }
                // LHS and RHS couldn't be interpreted, only simplified.
                ResolveExpressionResult::Modified(expr2) => ResolveExpressionResult::Modified(
                    Expression::BinaryOperation(Box::new(expr1), operator, Box::new(expr2)),
                ),
            },
        })
    }

    // Does not return an interpreted result, only a modified result. Since assignment *writes* to a
    // value instead of reads from it, there is no way to return an "interpreted" result.
    fn resolve_assignment_access_expression(
        &mut self,
        access_expression: &Expression,
    ) -> Result<Expression, CompileProblem> {
        Result::Ok(match access_expression {
            Expression::Variable(id) => Expression::Variable(self.convert(*id)),
            Expression::Access { base, indexes } => {
                let resolved_base = self.resolve_assignment_access_expression(base)?;
                let mut resolved_indexes = Vec::with_capacity(indexes.len());
                for index in indexes {
                    let resolved_index = self.resolve_expression(index)?;
                    resolved_indexes.push(match resolved_index {
                        ResolveExpressionResult::Interpreted(value) => Expression::Literal(value),
                        ResolveExpressionResult::Modified(expression) => expression,
                    });
                }
                Expression::Access {
                    base: Box::new(resolved_base),
                    indexes: resolved_indexes,
                }
            }
            _ => unreachable!("No other expression types found in assignment access expressions."),
        })
    }

    // Tries to assign known data to an access expression. The access expression must be
    // either a variable or an access expression. Indexes in the access expression can be
    // any kind of expression. If the value cannot be assigned at compile time, a modified
    // expression to assign the value at run time.
    fn assign_value_to_expression(
        &mut self,
        target: &Expression,
        value: KnownData,
    ) -> Result<Result<(), Expression>, CompileProblem> {
        Result::Ok(match target {
            Expression::Variable(id) => {
                self.program.set_temporary_value(self.convert(*id), value);
                Result::Ok(())
            }
            Expression::Access { base, indexes } => {
                let base_var_id = match base.borrow() {
                    Expression::Variable(id) => self.convert(*id),
                    _ => unreachable!("Nothing else can be the base of an access."),
                };
                let mut all_indexes_known = true;
                let mut known_indexes = Vec::new();
                let mut resolved_indexes = Vec::with_capacity(indexes.len());
                for index in indexes {
                    let result = self.resolve_expression(index)?;
                    match result {
                        ResolveExpressionResult::Interpreted(value) => {
                            if all_indexes_known {
                                if let KnownData::Int(value) = value {
                                    // TODO: Check that value is positive.
                                    known_indexes.push(value as usize);
                                } else {
                                    panic!("TODO: nice error, index must be int.");
                                }
                            } else {
                                resolved_indexes.push(Expression::Literal(value));
                            }
                        }
                        ResolveExpressionResult::Modified(expression) => {
                            resolved_indexes.push(expression);
                            all_indexes_known = false;
                        }
                    }
                }
                if all_indexes_known {
                    self.program
                        .borrow_variable_mut(base_var_id)
                        .borrow_temporary_value_mut()
                        .require_array_mut()
                        .set_item(&known_indexes, value);
                    Result::Ok(())
                } else {
                    Result::Err(Expression::Access {
                        base: Box::new(Expression::Variable(base_var_id)),
                        indexes: resolved_indexes,
                    })
                }
            }
            _ => unreachable!("Assignment target cannot contain any other expressions.")
        })
    }

    fn resolve_expression(
        &mut self,
        old_expression: &Expression,
    ) -> Result<ResolveExpressionResult, CompileProblem> {
        Result::Ok(match old_expression {
            Expression::Literal(value) => ResolveExpressionResult::Interpreted(value.clone()),
            Expression::Variable(id) => {
                let converted_id = self.convert(*id);
                let variable = self.program.borrow_variable(converted_id);
                let temporary_value = variable.borrow_temporary_value();
                match temporary_value {
                    KnownData::Unknown => {
                        ResolveExpressionResult::Modified(Expression::Variable(converted_id))
                    }
                    _ => ResolveExpressionResult::Interpreted(temporary_value.clone()),
                }
            }
            Expression::Access { base, indexes } => {
                // TODO: Optimize accessing values in big arrays so that we don't have to clone
                // the entire array to pick out one value.
                let resolved_base = self.resolve_expression(base)?;
                let mut resolved_indexes = Vec::with_capacity(indexes.len());
                for index in indexes {
                    resolved_indexes.push(self.resolve_expression(index)?);
                }
                self.resolve_access_expression(resolved_base, resolved_indexes)?
            }
            Expression::InlineReturn => unimplemented!(),

            Expression::UnaryOperation(..) => unimplemented!(),
            Expression::BinaryOperation(operand_1, operator, operand_2) => {
                let resolved_operand_1 = self.resolve_expression(operand_1)?;
                let resolved_operand_2 = self.resolve_expression(operand_2)?;
                self.resolve_binary_expression(resolved_operand_1, *operator, resolved_operand_2)?
            }

            Expression::Collect(values) => {
                let mut resolved_values = Vec::with_capacity(values.len());
                let mut all_known = true;
                for value in values {
                    let resolved_value = self.resolve_expression(value)?;
                    if let ResolveExpressionResult::Modified(..) = &resolved_value {
                        all_known = false;
                    }
                    resolved_values.push(resolved_value);
                }
                if all_known {
                    let mut data = Vec::with_capacity(resolved_values.len());
                    for value in resolved_values {
                        data.push(match value {
                            ResolveExpressionResult::Interpreted(data) => data,
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
                        ResolveExpressionResult::Interpreted(KnownData::Array(NVec::collect(
                            sub_arrays,
                        )))
                    } else {
                        // Each element is a scalar, so just make a new 1d array out of them.
                        ResolveExpressionResult::Interpreted(KnownData::Array(NVec::from_vec(data)))
                    }
                } else {
                    let mut items = Vec::with_capacity(resolved_values.len());
                    // Standard treatment, fully interpreted values become literal expressions.
                    for value in resolved_values.into_iter() {
                        match value {
                            ResolveExpressionResult::Interpreted(value) => {
                                items.push(Expression::Literal(value))
                            }
                            ResolveExpressionResult::Modified(expression) => items.push(expression),
                        }
                    }
                    ResolveExpressionResult::Modified(Expression::Collect(items))
                }
            }

            Expression::Assert(value) => {
                let resolved_value = self.resolve_expression(value)?;
                match resolved_value {
                    ResolveExpressionResult::Interpreted(value) => {
                        if let KnownData::Bool(succeed) = value {
                            if succeed {
                                ResolveExpressionResult::Interpreted(KnownData::Void)
                            } else {
                                panic!("TODO: nice error, assert guaranteed to fail.")
                            }
                        } else {
                            panic!("TODO: nice error, argument to assert is not bool.")
                        }
                    }
                    ResolveExpressionResult::Modified(expression) => {
                        ResolveExpressionResult::Modified(Expression::Assert(Box::new(expression)))
                    }
                }
            }
            Expression::Assign { target, value } => {
                match self.resolve_expression(value)? {
                    ResolveExpressionResult::Interpreted(known_value) => {
                        let result = self.assign_value_to_expression(target, known_value)?;
                        if let Result::Err(resolved_expresion) = result {
                            ResolveExpressionResult::Modified(resolved_expresion)
                        } else {
                            ResolveExpressionResult::Interpreted(KnownData::Void)
                        }
                    }
                    ResolveExpressionResult::Modified(resolved_value) => {
                        let resolved_target = self.resolve_assignment_access_expression(target)?;
                        ResolveExpressionResult::Modified(Expression::Assign {
                            target: Box::new(resolved_target),
                            value: Box::new(resolved_value),
                        })
                    }
                }
            }
            Expression::Return => ResolveExpressionResult::Modified(Expression::Return),

            Expression::FuncCall{..} => {
                unimplemented!()
            }
        })
    }

    fn resolve_scope(&mut self, source: ScopeId) -> Result<ScopeId, CompileProblem> {
        let copied = self.copy_scope(source, self.program.borrow_scope(source).get_parent())?;
        let old_body = self.program.clone_scope_body(source);
        for expression in old_body {
            let resolved = self.resolve_expression(&expression)?;
            match resolved {
                // If it was successfully interpreted, we don't need to do anything.
                ResolveExpressionResult::Interpreted(..) => (),
                ResolveExpressionResult::Modified(expression) => match expression {
                    // TODO: Warn for expressions that have outputs that are not stored.
                    Expression::Return => break,
                    _ => self.program.add_expression(copied, expression)
                }
            }
        }
        Result::Ok(copied)
    }

    fn entry_point(&mut self, entry_scope: ScopeId) -> Result<ScopeId, CompileProblem> {
        self.resolve_scope(entry_scope)
    }
}
