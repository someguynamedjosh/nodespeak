use super::{Content, ScopeSimplifier, SimplifiedExpression, problems};
use crate::problem::{CompileProblem, FilePosition};
use crate::vague::structure::{BaseType, BinaryOperator, DataType, Expression, FunctionData, KnownData};
use std::borrow::Borrow;

impl<'a> ScopeSimplifier<'a> {
    pub(super) fn simplify_access_expression(
        &mut self,
        simplified_base: SimplifiedExpression,
        mut base_position: FilePosition,
        simplified_indexes: Vec<(SimplifiedExpression, FilePosition)>,
        position: FilePosition,
    ) -> Result<SimplifiedExpression, CompileProblem> {
        let final_type = simplified_base
            .data_type
            .clone_and_unwrap(simplified_indexes.len());
        Result::Ok(match simplified_base.content {
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
                for (index, index_position) in simplified_indexes.into_iter() {
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
                        return Result::Ok(SimplifiedExpression {
                            content: Content::Interpreted(element),
                            data_type: final_type,
                        });
                    }
                }

                // If there are no extra dynamic indexes, we can just return whatever portion of the
                // data that we were able to collect at compile time.
                if dynamic_indexes.len() == 0 {
                    SimplifiedExpression {
                        content: Content::Interpreted(KnownData::Array(base_data)),
                        data_type: final_type,
                    }
                // Otherwise, make an access expression using the leftover dynamic indexes.
                } else {
                    SimplifiedExpression {
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
                let mut new_indexes = Vec::with_capacity(simplified_indexes.len());
                for (index, index_position) in simplified_indexes.into_iter() {
                    if !index.data_type.is_specific_scalar(&BaseType::Int) {
                        return Result::Err(problems::array_index_not_int(
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
                SimplifiedExpression {
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

    pub(super) fn simplify_binary_expression(
        &mut self,
        simplified_operand_1: SimplifiedExpression,
        operand_1_position: FilePosition,
        operator: BinaryOperator,
        simplified_operand_2: SimplifiedExpression,
        operand_2_position: FilePosition,
        position: FilePosition,
    ) -> Result<SimplifiedExpression, CompileProblem> {
        let result_type = match super::util::biggest_type(
            &simplified_operand_1.data_type,
            &simplified_operand_2.data_type,
            self.program,
        ) {
            Result::Ok(rtype) => rtype,
            Result::Err(..) => {
                return Result::Err(problems::no_bct(
                    position,
                    operand_1_position,
                    &simplified_operand_1.data_type,
                    operand_2_position,
                    &simplified_operand_2.data_type,
                ))
            }
        };
        // TODO: Generate proxies when necessary.
        let content = match simplified_operand_1.content {
            Content::Interpreted(dat1) => match simplified_operand_2.content {
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
            Content::Modified(expr1) => match simplified_operand_2.content {
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
        Result::Ok(SimplifiedExpression {
            content,
            data_type: result_type,
        })
    }

    pub(super) fn simplify_func_call(
        &mut self,
        function: &Expression,
        setup: &Vec<Expression>,
        teardown: &Vec<Expression>,
        position: &FilePosition,
    ) -> Result<SimplifiedExpression, CompileProblem> {
        // We want to make a new table specifically for this function, so that any variable
        // conversions we make won't be applied to other calls to the same funciton.
        self.push_table();
        // Make sure we can figure out what function is being called right now at compile time.
        let simplified_function = match self.simplify_expression(function.borrow())?.content {
            Content::Interpreted(value) => value,
            Content::Modified(..) => {
                return Result::Err(problems::vague_function(
                    position.clone(),
                    function.clone_position(),
                ))
            }
        };
        // Get the actual function data.
        let function_data = match simplified_function {
            KnownData::Function(data) => data,
            _ => return Result::Err(problems::not_function(function.clone_position())),
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
            match self.simplify_expression(expr)?.content {
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
            match self.simplify_expression(&expression)?.content {
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

        // Now try and simplify as much of the teardown as possible. Keep track of whether or not
        // there is an inline return statement and what value it contains.
        let mut runtime_teardown = Vec::new();
        let mut inline_output = None;
        for expr in teardown {
            match expr {
                Expression::InlineReturn(return_value, position) => {
                    let simplified = self.simplify_expression(return_value)?;
                    if let Content::Modified(new_expression) = &simplified.content {
                        runtime_teardown.push(Expression::InlineReturn(
                            Box::new(new_expression.clone()),
                            position.clone(),
                        ))
                    }
                    inline_output = Some(simplified);
                }
                _ => {
                    if let Content::Modified(new_expr) = self.simplify_expression(expr)?.content {
                        runtime_teardown.push(new_expr);
                    }
                }
            }
        }

        self.pop_table();

        Result::Ok(if empty_body && runtime_teardown.len() == 0 {
            match inline_output {
                Some(value) => value,
                None => SimplifiedExpression {
                    content: Content::Interpreted(KnownData::Void),
                    data_type: DataType::scalar(BaseType::Void),
                },
            }
        } else {
            SimplifiedExpression {
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
                    Some(simplified) => simplified.data_type,
                    None => DataType::scalar(BaseType::Void),
                },
            }
        })
    }
}