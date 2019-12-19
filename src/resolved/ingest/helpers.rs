use super::{BaseType, Content, DataType, ScopeSimplifier};
use crate::problem::{CompileProblem, FilePosition};
use crate::resolved::structure as o;
use crate::vague::structure as i;
use std::borrow::Borrow;

impl<'a> ScopeSimplifier<'a> {
    pub(super) fn resolve_automatic_type(
        &mut self,
        target: &o::Expression,
        data_type: &DataType,
    ) -> Result<(), CompileProblem> {
        match target {
            // TODO: Handle proxies.
            o::Expression::Access { base, indexes, .. } => {
                self.resolve_automatic_type(base, &data_type.clone_and_unwrap(indexes.len()))?;
            }
            // TODO: Handle arrays of automatic types.
            o::Expression::Variable(id, ..) => {
                self.target[*id].set_data_type(
                    data_type
                        .to_output_type()
                        .expect("TODO: Nice error, invalid type."),
                );
            }
            _ => unreachable!("Should not resolve automatic types of anything else."),
        }
        Result::Ok(())
    }

    // Tries to assign known data to an access expression. The access expression must be
    // either a variable or an access expression. Indexes in the access expression can be
    // any kind of expression. If the value cannot be assigned at compile time, a modified
    // expression to assign the value at run time will be returned instead.
    pub(super) fn assign_value_to_expression(
        &mut self,
        target: &i::Expression,
        value: i::KnownData,
        position: FilePosition,
    ) -> Result<Result<(), o::Expression>, CompileProblem> {
        Result::Ok(match target {
            i::Expression::Variable(id, ..) => {
                self.set_temporary_value(*id, value);
                Result::Ok(())
            }
            i::Expression::Access { base, indexes, .. } => {
                let (base_var_id, base_var_pos) = match base.borrow() {
                    i::Expression::Variable(id, position) => (*id, position.clone()),
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
                            if let i::KnownData::Int(value) = value {
                                assert!(
                                    value >= 0,
                                    "TODO: nice error, index must be non-negative."
                                );
                                if all_indexes_known {
                                    known_indexes.push(value as usize);
                                }
                                // Also put a literal into resolved indexes in case we don't know
                                // all the indexes and need to construct a runtime expression to set
                                // the value.
                                let data = o::KnownData::Int(value);
                                let literal = o::Expression::Literal(data, index_position);
                                resolved_indexes.push(literal);
                            } else {
                                panic!("TODO: nice error, index must be int.");
                            }
                        }
                        Content::Modified(expression) => {
                            resolved_indexes.push(expression);
                            all_indexes_known = false;
                        }
                    }
                }
                if all_indexes_known {
                    // TODO: Check that value is correct type.
                    // TODO: Handle setting array slices.
                    self.borrow_temporary_value_mut(base_var_id)
                        .require_array_mut()
                        .set_item(&known_indexes, value);
                    Result::Ok(())
                } else {
                    Result::Err(o::Expression::Access {
                        base: Box::new(
                            self.convert(base_var_id)
                                .expect("TODO: nice error, var not defined.")
                                .clone(),
                        ),
                        indexes: resolved_indexes,
                        position,
                    })
                }
            }
            _ => unreachable!("Assignment target cannot contain any other expressions."),
        })
    }

    pub(super) fn assign_unknown_to_expression(&mut self, target: &i::Expression) {
        match target {
            i::Expression::Variable(id, ..) => {
                self.set_temporary_value(*id, i::KnownData::Unknown);
            }
            i::Expression::Access { base, .. } => {
                // TODO: Something more efficient that takes into account any known indexes.
                let (base_var_id, _base_var_pos) = match base.borrow() {
                    i::Expression::Variable(id, position) => (id, position.clone()),
                    _ => unreachable!("Nothing else can be the base of an access."),
                };
                self.set_temporary_value(*base_var_id, i::KnownData::Unknown);
            }
            _ => unreachable!("Cannot have anything else in an assignment expression."),
        }
    }

    pub(super) fn resolve_assignment_access_expression(
        &mut self,
        access_expression: &i::Expression,
        data_type: o::DataType,
    ) -> Result<(o::Expression, DataType), CompileProblem> {
        self.resolve_assignment_access_impl(access_expression, data_type, 0)
    }

    // Does not return an interpreted result, only a modified result. Since assignment *writes* to a
    // value instead of reads from it, there is no way to return an "interpreted" result.
    pub(super) fn resolve_assignment_access_impl(
        &mut self,
        access_expression: &i::Expression,
        data_type: o::DataType,
        num_indexes: usize,
    ) -> Result<(o::Expression, DataType), CompileProblem> {
        // TODO: More complicated automatic types like [4]Auto or something.
        Result::Ok(match access_expression {
            i::Expression::Variable(id, position) => {
                if let Some(new_expr) = self.convert(*id) {
                    let cloned = new_expr.clone();
                    let data_type = DataType::from_output_type(&cloned.get_type(&self.target));
                    (cloned, data_type)
                } else if self.is_unresolved_auto_var(*id) {
                    if (num_indexes > 0) {
                        panic!("TODO: Automatic array types.");
                    }
                    self.mark_auto_var_resolved(*id);
                    let inter_type = DataType::from_output_type(&data_type);
                    let var = o::Variable::new(FilePosition::placeholder(), data_type);
                    let new_id = self
                        .target
                        .adopt_and_define_intermediate(self.current_scope, var);
                    let expr = o::Expression::Variable(new_id, position.clone());
                    self.add_conversion(*id, expr.clone());
                    (expr, inter_type)
                } else {
                    panic!("TODO: Nice error, variable not defined.")
                }
            }
            i::Expression::Access {
                base,
                indexes,
                position,
            } => {
                let resolved_base =
                    self.resolve_assignment_access_impl(base, data_type, indexes.len())?;
                let mut resolved_indexes = Vec::with_capacity(indexes.len());
                for index in indexes {
                    let index_position = index.clone_position();
                    let resolved_index = self.resolve_expression(index)?;
                    resolved_indexes.push(match resolved_index.content {
                        Content::Interpreted(value) => {
                            if let i::KnownData::Int(index) = value {
                                assert!(index >= 0, "TODO: nice error, index must be nonnegative");
                                let data = o::KnownData::Int(index);
                                o::Expression::Literal(data, index_position)
                            } else {
                                panic!("TODO: nice error, array index must be integer.");
                            }
                        }
                        Content::Modified(expression) => expression,
                    });
                }
                let num_indexes = resolved_indexes.len();
                let expr = o::Expression::Access {
                    base: Box::new(resolved_base.0),
                    indexes: resolved_indexes,
                    position: position.clone(),
                };
                (expr, resolved_base.1.clone_and_unwrap(num_indexes))
            }
            i::Expression::InlineReturn(position) => (
                o::Expression::InlineReturn(position.clone()),
                DataType::scalar(BaseType::Automatic),
            ),
            _ => unreachable!("No other expression types found in assignment access expressions."),
        })
    }
}
