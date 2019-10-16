use super::{Content, ScopeSimplifier};
use crate::problem::{CompileProblem, FilePosition};
use crate::structure::{BaseType, DataType, Expression, FunctionData, KnownData, VariableId};
use std::borrow::Borrow;

impl<'a> ScopeSimplifier<'a> {
    pub(super) fn require_simplified_function_data(
        &self,
        from: VariableId,
    ) -> Result<FunctionData, CompileProblem> {
        match self.program[from].borrow_temporary_value() {
            KnownData::Unknown => panic!("TODO: nice error, vague function variable."),
            KnownData::Function(data) => Result::Ok(data.clone()),
            _ => panic!("TODO: variable does not contain a function."),
        }
    }

    pub(super) fn simplify_automatic_type(
        &mut self,
        target: &Expression,
        data_type: &DataType,
    ) -> Result<(), CompileProblem> {
        match target {
            Expression::Access { base, indexes, .. } => {
                self.simplify_automatic_type(base, &data_type.clone_and_unwrap(indexes.len()))?;
            }
            // TODO: Handle arrays of automatic types.
            Expression::Variable(id, ..) => {
                self.program[*id].set_data_type(data_type.clone());
            }
            _ => unreachable!("Should not simplify automatic types of anything else."),
        }
        Result::Ok(())
    }

    // Tries to assign known data to an access expression. The access expression must be
    // either a variable or an access expression. Indexes in the access expression can be
    // any kind of expression. If the value cannot be assigned at compile time, a modified
    // expression to assign the value at run time will be returned instead.
    pub(super) fn assign_value_to_expression(
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
                let mut simplified_indexes = Vec::with_capacity(indexes.len());
                for index in indexes {
                    let index_position = index.clone_position();
                    let result = self.simplify_expression(index)?;
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
                            // Also put a literal into simplified indexes in case we don't know all
                            // the indexes and need to construct a runtime expression to set the
                            // value.
                            simplified_indexes.push(Expression::Literal(value, index_position));
                        }
                        Content::Modified(expression) => {
                            simplified_indexes.push(expression);
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
                        indexes: simplified_indexes,
                        position,
                    })
                }
            }
            _ => unreachable!("Assignment target cannot contain any other expressions."),
        })
    }

    pub(super) fn assign_unknown_to_expression(&mut self, target: &Expression) {
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
    pub(super) fn simplify_assignment_access_expression(
        &mut self,
        access_expression: &Expression,
    ) -> Result<(Expression, DataType), CompileProblem> {
        Result::Ok(match access_expression {
            Expression::Variable(id, position) => (
                Expression::Variable(self.convert(*id), position.clone()),
                self.program[self.convert(*id)].borrow_data_type().clone(),
            ),
            Expression::PickInput(function, index, position) => {
                let scope = self.require_simplified_function_data(*function)?.get_body();
                let converted = self.convert(self.program[scope].get_input(*index));
                (
                    Expression::Variable(converted, position.clone()),
                    self.program[converted].borrow_data_type().clone(),
                )
            }
            Expression::PickOutput(function, index, position) => {
                let scope = self.require_simplified_function_data(*function)?.get_body();
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
                let simplified_base = self.simplify_assignment_access_expression(base)?;
                let mut simplified_indexes = Vec::with_capacity(indexes.len());
                for index in indexes {
                    let index_position = index.clone_position();
                    let simplified_index = self.simplify_expression(index)?;
                    simplified_indexes.push(match simplified_index.content {
                        Content::Interpreted(value) => Expression::Literal(value, index_position),
                        Content::Modified(expression) => expression,
                    });
                }
                let num_indexes = simplified_indexes.len();
                let expr = Expression::Access {
                    base: Box::new(simplified_base.0),
                    indexes: simplified_indexes,
                    position: position.clone(),
                };
                (expr, simplified_base.1.clone_and_unwrap(num_indexes))
            }
            Expression::InlineReturn(..) => (
                access_expression.clone(),
                DataType::scalar(BaseType::Automatic),
            ),
            _ => unreachable!("No other expression types found in assignment access expressions."),
        })
    }
}
