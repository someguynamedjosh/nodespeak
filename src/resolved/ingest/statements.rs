use super::{problems, BaseType, Content, DataType, ScopeSimplifier, SimplifiedExpression};
use crate::problem::{CompileProblem, FilePosition};
use crate::resolved::structure as o;
use crate::vague::structure as i;

impl<'a> ScopeSimplifier<'a> {
    pub(super) fn resolve_creation_point(
        &mut self,
        old_var_id: i::VariableId,
        position: &FilePosition,
    ) -> Result<SimplifiedExpression, CompileProblem> {
        let old_data_type = self.source[old_var_id].borrow_data_type().clone();
        let intermediate_data_type = self.input_to_intermediate_type(old_data_type)?;
        if let Result::Ok(output_type) = intermediate_data_type.to_output_type() {
            let new_var = o::Variable::new(position.clone(), output_type);
            let var_id = self
                .target
                .adopt_and_define_intermediate(self.current_scope, new_var);
            let expression = o::Expression::Variable(var_id, FilePosition::placeholder());
            self.add_conversion(old_var_id, expression);
        } else if intermediate_data_type.is_automatic() {
            self.add_unresolved_auto_var(old_var_id);
        }
        let mut starting_value = self.source[old_var_id].borrow_initial_value().clone();
        if let i::KnownData::Unknown = starting_value {
            if intermediate_data_type.borrow_dimensions().len() > 0 {
                let dims = intermediate_data_type.borrow_dimensions();
                starting_value = i::KnownData::build_array(dims);
            }
        }
        self.set_temporary_value(old_var_id, starting_value);

        Result::Ok(SimplifiedExpression {
            content: Content::Interpreted(i::KnownData::Void),
            data_type: DataType::new(BaseType::Void),
        })
    }

    pub(super) fn resolve_assign_statement(
        &mut self,
        target: &i::Expression,
        value: &i::Expression,
        position: &FilePosition,
    ) -> Result<SimplifiedExpression, CompileProblem> {
        let value_pos = value.clone_position();
        let resolved_value = self.resolve_expression(value)?;
        let data_type = match resolved_value.data_type.to_output_type() {
            Result::Ok(result) => result,
            Result::Err(..) => panic!("TODO: Nice error."),
        };
        let target_pos = target.clone_position();
        let resolved_target = self.resolve_assignment_access_expression(target, data_type)?;
        if !resolved_value.data_type.equivalent(&resolved_target.1) {
            return Err(problems::mismatched_assign(
                position.clone(),
                target_pos,
                &resolved_target.1,
                value_pos,
                &resolved_value.data_type,
            ));
        }
        Result::Ok(match resolved_value.content {
            Content::Interpreted(known_value) => {
                let result =
                    self.assign_value_to_expression(&target, known_value, position.clone())?;
                if let Result::Err(resolved_expresion) = result {
                    self.assign_unknown_to_expression(&target);
                    SimplifiedExpression {
                        content: Content::Modified(resolved_expresion),
                        data_type: DataType::scalar(BaseType::Void),
                    }
                } else {
                    SimplifiedExpression {
                        content: Content::Interpreted(i::KnownData::Void),
                        data_type: DataType::scalar(BaseType::Void),
                    }
                }
            }
            Content::Modified(resolved_value) => {
                self.assign_unknown_to_expression(&target);
                let content = Content::Modified(o::Expression::Assign {
                    target: Box::new(resolved_target.0),
                    value: Box::new(resolved_value),
                    position: position.clone(),
                });
                SimplifiedExpression {
                    content,
                    data_type: DataType::scalar(BaseType::Void),
                }
            }
        })
    }

    pub(super) fn resolve_assert_statement(
        &mut self,
        value: &i::Expression,
        position: &FilePosition,
    ) -> Result<SimplifiedExpression, CompileProblem> {
        let resolved_value = self.resolve_expression(value)?;
        Result::Ok(match resolved_value.content {
            Content::Interpreted(value) => {
                if let i::KnownData::Bool(succeed) = value {
                    if succeed {
                        SimplifiedExpression {
                            content: Content::Interpreted(i::KnownData::Void),
                            data_type: DataType::scalar(BaseType::Void),
                        }
                    } else {
                        return Result::Err(problems::guaranteed_assert(position.clone()));
                    }
                } else {
                    panic!("TODO: nice error, argument to assert is not bool.")
                }
            }
            Content::Modified(expression) => SimplifiedExpression {
                content: Content::Modified(o::Expression::Assert(
                    Box::new(expression),
                    position.clone(),
                )),
                data_type: DataType::scalar(BaseType::Void),
            },
        })
    }
}
