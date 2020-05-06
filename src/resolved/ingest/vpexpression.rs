use super::{
    problems, DataType, PossiblyKnownData, ResolvedStatement, ResolvedVPExpression, ScopeResolver,
};
use crate::high_level::problem::{CompileProblem, FilePosition};
use crate::resolved::structure as o;
use crate::vague::structure as i;

impl<'a> ScopeResolver<'a> {
    fn resolve_vp_variable(
        &mut self,
        var_id: i::VariableId,
        position: &FilePosition,
    ) -> Result<ResolvedVPExpression, CompileProblem> {
        let value = self.borrow_temporary_value(var_id);
        if let Ok(kvalue) = value.to_known_data() {
            let typ = self.resolve_data_type_partially(kvalue.get_data_type());
            return Ok(ResolvedVPExpression::Interpreted(
                kvalue,
                position.clone(),
                typ,
            ));
        }
        let (resolved_id, dtype) = self
            .get_var_info(var_id)
            .expect("Variable used before defined, should have been caught by the previous phase.");
        if dtype.is_automatic() {
            panic!(
                "TODO: Nice error, data type of automatic variable has not been determined yet."
            );
        }
        let resolved_id = resolved_id.expect("TODO: Nice error, cannot use var at run time.");
        Ok(ResolvedVPExpression::Modified(
            o::VPExpression::Variable(resolved_id, position.clone()),
            dtype.clone(),
        ))
    }

    fn resolve_collect(
        &mut self,
        items: &Vec<i::VPExpression>,
        position: &FilePosition,
    ) -> Result<ResolvedVPExpression, CompileProblem> {
        debug_assert!(items.len() > 0);
        let mut resolved_items = Vec::new();
        for item in items {
            resolved_items.push(self.resolve_vp_expression(item)?);
        }
        let typ = resolved_items[0].borrow_data_type();
        let mut all_known = true;
        for item in &resolved_items {
            if item.borrow_data_type() != typ {
                panic!("TODO: Nice error, items in collect are not compatible.");
            }
            if let ResolvedVPExpression::Modified(..) = item {
                all_known = false;
            }
        }

        let atype = DataType::Array(resolved_items.len(), Box::new(typ.clone()));
        Ok(if all_known {
            let mut data_items = Vec::new();
            for item in resolved_items {
                if let ResolvedVPExpression::Interpreted(data, ..) = item {
                    data_items.push(data);
                } else {
                    unreachable!("Already checked that all items are known.");
                }
            }
            ResolvedVPExpression::Interpreted(
                i::KnownData::Array(data_items),
                position.clone(),
                atype,
            )
        } else {
            let mut vp_exprs = Vec::new();
            for item in resolved_items {
                vp_exprs.push(item.as_vp_expression()?)
            }
            ResolvedVPExpression::Modified(
                o::VPExpression::Collect(vp_exprs, position.clone()),
                atype,
            )
        })
    }

    fn resolve_build_array_type(
        &mut self,
        dimensions: &Vec<i::VPExpression>,
        base: &i::VPExpression,
        position: &FilePosition,
    ) -> Result<ResolvedVPExpression, CompileProblem> {
        let mut int_dims = Vec::new();
        for dim in dimensions.iter().rev() {
            let resolved = self.resolve_vp_expression(dim)?;
            let rpos = resolved.clone_position();
            if let ResolvedVPExpression::Interpreted(data, ..) = resolved {
                if let i::KnownData::Int(value) = data {
                    if value < 1 {
                        return Err(problems::array_size_less_than_one(rpos, value));
                    }
                    int_dims.push(value as usize);
                } else {
                    return Err(problems::array_size_not_int(rpos, &data.get_data_type()));
                }
            } else {
                return Err(problems::array_size_not_resolved(rpos));
            }
        }
        let resolved_base = self.resolve_vp_expression(base)?;
        if resolved_base.borrow_data_type() != &DataType::DataType {
            return Err(problems::array_base_not_data_type(
                resolved_base.clone_position(),
                resolved_base.borrow_data_type(),
            ));
        }
        if let ResolvedVPExpression::Interpreted(data, ..) = resolved_base {
            if let i::KnownData::DataType(dtype, ..) = data {
                let mut final_type = dtype.clone();
                for dim in int_dims {
                    final_type = i::DataType::Array(dim, Box::new(final_type));
                }
                Ok(ResolvedVPExpression::Interpreted(
                    i::KnownData::DataType(final_type),
                    position.clone(),
                    DataType::DataType,
                ))
            } else {
                unreachable!("We already checked that the expr was a data type.")
            }
        } else {
            unreachable!("Data is both dynamic and a data type. That should be impossible.");
        }
    }

    fn resolve_vp_index_impl(
        &mut self,
        resolved_base: ResolvedVPExpression,
        index: &i::VPExpression,
    ) -> Result<ResolvedVPExpression, CompileProblem> {
        let resolved_index = self.resolve_vp_expression(index)?;
        let (array_length, etype) =
            if let DataType::Array(len, dtype) = resolved_base.borrow_data_type() {
                (*len, *(dtype.clone()))
            } else {
                panic!("TODO: Nice error, cannot index non-array.")
            };
        if resolved_index.borrow_data_type() != &DataType::Int {
            panic!("TODO: Nice error, array index not integer.")
        }

        match resolved_index {
            // If the index is compile-time constant.
            ResolvedVPExpression::Interpreted(data, index_pos, dtype) => {
                // Safe because we already checked that it was an int.
                let value = data.require_int();
                // Check that the index is in bounds.
                if value < 0 {
                    panic!("TODO: Nice error, index less than zero.");
                }
                if value as usize >= array_length {
                    panic!("TODO: Nice error, index too big.");
                }
                Ok(match resolved_base {
                    // If the base is also compile-time constant, return a new constant value.
                    ResolvedVPExpression::Interpreted(base_data, base_pos, base_type) => {
                        let element = base_data.require_array()[value as usize].clone();
                        let mut pos = base_pos;
                        pos.include_other(&index_pos);
                        ResolvedVPExpression::Interpreted(element, pos, etype)
                    }
                    ResolvedVPExpression::Modified(base_expr, base_type) => {
                        let pos = FilePosition::union(&[&base_expr.clone_position(), &index_pos]);
                        let index = Self::int_literal(value, index_pos);
                        ResolvedVPExpression::Modified(
                            // Otherwise, if it's an index expression, add the new index to it.
                            if let o::VPExpression::Index {
                                base,
                                mut indexes,
                                position,
                            } = base_expr
                            {
                                indexes.push(index);
                                o::VPExpression::Index {
                                    base,
                                    indexes,
                                    position: pos,
                                }
                            // Otherwise, make an entirely new index expression.
                            } else {
                                o::VPExpression::Index {
                                    base: Box::new(base_expr),
                                    indexes: vec![index],
                                    position: pos,
                                }
                            },
                            etype,
                        )
                    }
                })
            }
            // Otherwise, if the index is only available as a run-time expression...
            ResolvedVPExpression::Modified(index_expr, ..) => {
                let pos = FilePosition::union(&[
                    &resolved_base.clone_position(),
                    &index_expr.clone_position(),
                ]);
                let expr = match resolved_base {
                    // If the base is a compile-time constant, make it a literal and return a new
                    // expression.
                    ResolvedVPExpression::Interpreted(base_data, base_pos, base_type) => {
                        let base_expr = o::VPExpression::Literal(
                            Self::resolve_known_data(&base_data).expect("TODO: Nice error."),
                            base_pos,
                        );
                        o::VPExpression::Index {
                            base: Box::new(base_expr),
                            indexes: vec![index_expr],
                            position: pos,
                        }
                    }
                    ResolvedVPExpression::Modified(base_expr, base_type) => {
                        // Otherwise, if it's an index expression, add the new index to it.
                        if let o::VPExpression::Index {
                            base, mut indexes, ..
                        } = base_expr
                        {
                            indexes.push(index_expr);
                            o::VPExpression::Index {
                                base,
                                indexes,
                                position: pos,
                            }
                        // Otherwise, make an entirely new index expression.
                        } else {
                            o::VPExpression::Index {
                                base: Box::new(base_expr),
                                indexes: vec![index_expr],
                                position: pos,
                            }
                        }
                    }
                };
                Ok(ResolvedVPExpression::Modified(expr, etype))
            }
        }
    }

    fn resolve_unary_operation(
        &mut self,
        op: i::UnaryOperator,
        rhs: &i::VPExpression,
        position: &FilePosition,
    ) -> Result<ResolvedVPExpression, CompileProblem> {
        let res_rhs = self.resolve_vp_expression(rhs)?;
        // TODO: Check that the operand has a data type compatible with the operator.
        Ok(
            if let ResolvedVPExpression::Interpreted(data, pos, typ) = res_rhs {
                ResolvedVPExpression::Interpreted(
                    Self::compute_unary_operation(op, &data),
                    pos,
                    typ,
                )
            } else {
                let res_op = match op {
                    i::UnaryOperator::BNot => o::UnaryOperator::BNot,
                    i::UnaryOperator::Negate => o::UnaryOperator::Negate,
                    i::UnaryOperator::Not => o::UnaryOperator::Not,
                    i::UnaryOperator::Reciprocal => o::UnaryOperator::Reciprocal,
                    i::UnaryOperator::Sine => o::UnaryOperator::Sine,
                    i::UnaryOperator::Cosine => o::UnaryOperator::Cosine,
                    i::UnaryOperator::SquareRoot => o::UnaryOperator::SquareRoot,
                    i::UnaryOperator::Exp => o::UnaryOperator::Exp,
                    i::UnaryOperator::Exp2 => o::UnaryOperator::Exp2,
                    i::UnaryOperator::Log => o::UnaryOperator::Log,
                    i::UnaryOperator::Log10 => o::UnaryOperator::Log10,
                    i::UnaryOperator::Log2 => o::UnaryOperator::Log2,
                    i::UnaryOperator::Absolute => o::UnaryOperator::Absolute,
                    i::UnaryOperator::Floor => o::UnaryOperator::Floor,
                    i::UnaryOperator::Ceiling => o::UnaryOperator::Ceiling,
                    i::UnaryOperator::Truncate => o::UnaryOperator::Truncate,
                };
                let dtype = res_rhs.borrow_data_type().clone();
                ResolvedVPExpression::Modified(
                    o::VPExpression::UnaryOperation(
                        res_op,
                        Box::new(res_rhs.as_vp_expression()?),
                        position.clone(),
                    ),
                    dtype,
                )
            },
        )
    }

    fn resolve_binary_operation(
        &mut self,
        lhs: &i::VPExpression,
        operator: i::BinaryOperator,
        rhs: &i::VPExpression,
        position: &FilePosition,
    ) -> Result<ResolvedVPExpression, CompileProblem> {
        // TODO: Check that the operand has a data type compatible with the operator.
        let res_lhs = self.resolve_vp_expression(lhs)?;
        let res_rhs = self.resolve_vp_expression(rhs)?;
        let bct = if let Ok(bct) =
            Self::biggest_type(res_lhs.borrow_data_type(), res_rhs.borrow_data_type())
        {
            bct
        } else {
            return Err(problems::no_bct(
                position.clone(),
                lhs.clone_position(),
                res_lhs.borrow_data_type(),
                rhs.clone_position(),
                res_rhs.borrow_data_type(),
            ));
        };
        let bct = match operator {
            i::BinaryOperator::LessThan
            | i::BinaryOperator::LessThanOrEqual
            | i::BinaryOperator::GreaterThan
            | i::BinaryOperator::GreaterThanOrEqual
            | i::BinaryOperator::Equal
            | i::BinaryOperator::NotEqual => {
                let bct_dims = bct.collect_dims();
                DataType::make_array(&bct_dims[..], DataType::Bool)
            }
            _ => bct,
        };
        if let (
            ResolvedVPExpression::Interpreted(lhs_data, ..),
            ResolvedVPExpression::Interpreted(rhs_data, ..),
        ) = (&res_lhs, &res_rhs)
        {
            let result = Self::compute_binary_operation(lhs_data, operator, rhs_data);
            debug_assert!(self.resolve_data_type_partially(result.get_data_type()) == bct);
            Ok(ResolvedVPExpression::Interpreted(
                result,
                position.clone(),
                bct,
            ))
        } else {
            Ok(ResolvedVPExpression::Modified(
                o::VPExpression::BinaryOperation {
                    lhs: Box::new(res_lhs.as_vp_expression()?),
                    op: Self::resolve_operator(operator),
                    rhs: Box::new(res_rhs.as_vp_expression()?),
                    typ: bct
                        .clone()
                        .resolve()
                        .expect("TODO: Nice error, data not available at compile time."),
                    position: position.clone(),
                },
                bct,
            ))
        }
    }

    fn resolve_vp_index(
        &mut self,
        base: &i::VPExpression,
        indexes: &Vec<i::VPExpression>,
        position: &FilePosition,
    ) -> Result<ResolvedVPExpression, CompileProblem> {
        // Resolve the base expression that is being indexed.
        let resolved_base = self.resolve_vp_expression(base)?;
        let mut result = resolved_base;
        for index in indexes {
            result = self.resolve_vp_index_impl(result, index)?;
        }
        Ok(result)
    }

    fn resolve_macro_call(
        &mut self,
        mcro: &i::VPExpression,
        inputs: &Vec<i::VPExpression>,
        outputs: &Vec<i::FuncCallOutput>,
        position: &FilePosition,
    ) -> Result<ResolvedVPExpression, CompileProblem> {
        // Find out what macro we are calling.
        let rmacro = self.resolve_vp_expression(mcro)?;
        let macro_data = if let ResolvedVPExpression::Interpreted(data, ..) = rmacro {
            if let i::KnownData::Macro(data) = data {
                data
            } else {
                panic!("TODO: Nice error, not macro.");
            }
        } else {
            panic!("TODO: Nice error, vague macro.");
        };
        let body_scope = macro_data.get_body();
        let rscope = self.target.create_scope();
        let old_scope = self.current_scope;
        self.current_scope = rscope;
        self.push_table();

        // Copy each input value to a new variable. If we know what the input value is at compile
        // time, then just set its temporary value without creating an actual variable for it.
        let macro_inputs = self.source[body_scope].borrow_inputs().clone();
        if inputs.len() != macro_inputs.len() {
            panic!("TODO: Nice error, wrong number of inputs.");
        }
        for index in 0..macro_inputs.len() {
            let rinput = self.resolve_vp_expression(&inputs[index])?;
            let input_id = macro_inputs[index];
            if let ResolvedVPExpression::Interpreted(data, _, dtype) = rinput {
                self.set_var_info(input_id, None, dtype);
                self.set_temporary_value(input_id, PossiblyKnownData::from_known_data(&data));
            } else if let ResolvedVPExpression::Modified(rinput, dtype) = rinput {
                let pos = rinput.clone_position();
                // It is impossible to get a dynamic expression that returns compile time only data.
                let odtype = dtype.resolve().unwrap();
                let input_in_body = o::Variable::new(pos.clone(), odtype);
                let input_in_body_id = self.target.adopt_variable(input_in_body);
                self.set_var_info(macro_inputs[index], Some(input_in_body_id), dtype);
                self.target[self.current_scope].add_statement(o::Statement::Assign {
                    target: Box::new(o::VCExpression::variable(input_in_body_id, pos.clone())),
                    value: Box::new(rinput),
                    position: pos.clone(),
                });
            }
        }

        // Resolve all the statements into the new body.
        for statement in self.source[body_scope].borrow_body().clone() {
            if let ResolvedStatement::Modified(statement) = self.resolve_statement(&statement)? {
                self.target[rscope].add_statement(statement);
            }
        }

        // Copy all the output values to the VCEs given in the macro call.
        let mut inline_return = None;
        let macro_outputs = self.source[body_scope].borrow_outputs().clone();
        if outputs.len() != macro_outputs.len() {
            panic!("TODO: Nice error, wrong number of outputs.");
        }
        for index in 0..macro_outputs.len() {
            match &outputs[index] {
                i::FuncCallOutput::InlineReturn(..) => {
                    debug_assert!(inline_return.is_none());
                    inline_return = Some(macro_outputs[index]);
                }
                i::FuncCallOutput::VCExpression(vce) => {
                    let pos = vce.clone_position();
                    // This will handle all the icky optiization stuff for us.
                    let rs = self.resolve_assign_statement(
                        &vce,
                        &i::VPExpression::Variable(macro_outputs[index], pos.clone()),
                        &pos,
                    )?;
                    if let ResolvedStatement::Modified(news) = rs {
                        self.target[self.current_scope].add_statement(news);
                    }
                }
            }
        }

        let result = if let Some(output_var) = inline_return {
            // Undefined output should be caught by earlier phase.
            let dtype = self.get_var_info(output_var).unwrap().1.clone();
            let pkd = self.borrow_temporary_value(output_var);
            if let Ok(known_data) = pkd.to_known_data() {
                ResolvedVPExpression::Interpreted(known_data, position.clone(), dtype)
            } else {
                // A variable cannot carry an indeterminate value while being compile-time only.
                let var_id = self.get_var_info(output_var).unwrap().0.unwrap();
                ResolvedVPExpression::Modified(
                    o::VPExpression::Variable(var_id, position.clone()),
                    dtype,
                )
            }
        } else {
            ResolvedVPExpression::Interpreted(i::KnownData::Void, position.clone(), DataType::Void)
        };

        self.pop_table();
        self.current_scope = old_scope;
        // Add a statement to call the body we just made.
        self.target[self.current_scope].add_statement(o::Statement::MacroCall {
            mcro: rscope,
            position: position.clone(),
        });

        Ok(result)
    }

    pub(super) fn resolve_vp_expression(
        &mut self,
        input: &i::VPExpression,
    ) -> Result<ResolvedVPExpression, CompileProblem> {
        Ok(match input {
            i::VPExpression::Literal(value, pos) => ResolvedVPExpression::Interpreted(
                value.clone(),
                pos.clone(),
                self.resolve_data_type_partially(value.get_data_type()),
            ),
            i::VPExpression::Variable(id, position) => self.resolve_vp_variable(*id, position)?,
            i::VPExpression::Collect(items, position) => self.resolve_collect(items, position)?,
            i::VPExpression::BuildArrayType {
                dimensions,
                base,
                position,
            } => self.resolve_build_array_type(dimensions, base, position)?,

            i::VPExpression::UnaryOperation(op, a, position) => {
                self.resolve_unary_operation(*op, a, position)?
            }
            i::VPExpression::BinaryOperation(lhs, operator, rhs, position) => {
                self.resolve_binary_operation(lhs, *operator, rhs, position)?
            }
            i::VPExpression::Index {
                base,
                indexes,
                position,
            } => self.resolve_vp_index(base, indexes, position)?,
            i::VPExpression::MacroCall {
                mcro,
                inputs,
                outputs,
                position,
            } => self.resolve_macro_call(mcro, inputs, outputs, position)?,
        })
    }
}
