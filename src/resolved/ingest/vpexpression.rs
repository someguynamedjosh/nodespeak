use super::{problems, DataType, ScopeResolver, ResolvedVPExpression};
use crate::problem::{CompileProblem, FilePosition};
use crate::resolved::structure as o;
use crate::vague::structure as i;
use std::borrow::Borrow;

impl<'a> ScopeResolver<'a> {
    fn resolve_vp_variable(
        &mut self,
        var_id: i::VariableId,
        position: &FilePosition,
    ) -> Result<ResolvedVPExpression, CompileProblem> {
        if let Some(value) = self.borrow_temporary_value(var_id) {
            Ok(ResolvedVPExpression::Interpreted(
                value.clone(),
                position.clone(),
                self.resolve_data_type_partially(value.get_data_type()),
            ))
        } else {
            let (resolved_id, dtype) = self.get_var_info(var_id).expect(
                "Variable used before defined, should have been caught by the previous phase.",
            );
            let resolved_id = resolved_id.expect("TODO: Nice error, cannot use var at run time.");
            Ok(ResolvedVPExpression::Modified(
                o::VPExpression::Variable(resolved_id, position.clone()),
                dtype.clone(),
            ))
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

    fn resolve_binary_operation(
        &mut self,
        lhs: &i::VPExpression,
        operator: i::BinaryOperator,
        rhs: &i::VPExpression,
        position: &FilePosition,
    ) -> Result<ResolvedVPExpression, CompileProblem> {
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
        if let (
            ResolvedVPExpression::Interpreted(lhs_data, ..),
            ResolvedVPExpression::Interpreted(rhs_data, ..),
        ) = (&res_lhs, &res_rhs)
        {
            let result = Self::compute_binary_operation(lhs_data, operator, rhs_data);
            debug_assert!(self.resolve_data_type_partially(result.get_data_type()) == bct);
            Ok(ResolvedVPExpression::Interpreted(result, position.clone(), bct))
        } else {
            Ok(ResolvedVPExpression::Modified(
                o::VPExpression::BinaryOperation(
                    Box::new(res_lhs.as_vp_expression()?),
                    Self::resolve_operator(operator),
                    Box::new(res_rhs.as_vp_expression()?),
                    position.clone()
                ),
                bct
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
            i::VPExpression::Collect(..) => unimplemented!(),
            i::VPExpression::BuildArrayType { .. } => unimplemented!(),

            i::VPExpression::UnaryOperation(..) => unimplemented!(),
            i::VPExpression::BinaryOperation(lhs, operator, rhs, position) => {
                self.resolve_binary_operation(lhs, *operator, rhs, position)?
            }
            i::VPExpression::Index {
                base,
                indexes,
                position,
            } => self.resolve_vp_index(base, indexes, position)?,
            i::VPExpression::FuncCall { .. } => unimplemented!(),
        })
    }
}
