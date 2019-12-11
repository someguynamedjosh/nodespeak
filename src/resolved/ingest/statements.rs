use super::{problems, BaseType, Content, DataType, ScopeSimplifier, SimplifiedExpression};
use crate::problem::{CompileProblem, FilePosition};
use crate::resolved::structure as o;
use crate::vague::structure as i;

impl<'a> ScopeSimplifier<'a> {
    pub(super) fn simplify_creation_point(
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
        }
        self.set_temporary_value(
            old_var_id,
            self.source[old_var_id].borrow_initial_value().clone(),
        );

        Result::Ok(SimplifiedExpression {
            content: Content::Interpreted(i::KnownData::Void),
            data_type: DataType::new(BaseType::Void),
        })
    }

    pub(super) fn simplify_assign_statement(
        &mut self,
        target: &i::Expression,
        value: &i::Expression,
        position: &FilePosition,
    ) -> Result<SimplifiedExpression, CompileProblem> {
        // TODO: This method can probably be cleaner / more efficient.
        let simplified_value = self.simplify_expression(value)?;
        let simplified_target = self.simplify_assignment_access_expression(target)?;
        if simplified_target.1.is_automatic() {
            self.simplify_automatic_type(&simplified_target.0, &simplified_value.data_type)?;
        } else if !simplified_value.data_type.equivalent(&simplified_target.1) {
            panic!("TODO: nice error, mismatched data types in assignment.");
        }
        Result::Ok(match simplified_value.content {
            Content::Interpreted(known_value) => {
                let result =
                    self.assign_value_to_expression(&target, known_value, position.clone())?;
                if let Result::Err(simplified_expresion) = result {
                    SimplifiedExpression {
                        content: Content::Modified(simplified_expresion),
                        data_type: DataType::scalar(BaseType::Void),
                    }
                } else {
                    SimplifiedExpression {
                        content: Content::Interpreted(i::KnownData::Void),
                        data_type: DataType::scalar(BaseType::Void),
                    }
                }
            }
            Content::Modified(simplified_value) => {
                self.assign_unknown_to_expression(&target);
                let content = Content::Modified(o::Expression::Assign {
                    target: Box::new(simplified_target.0),
                    value: Box::new(simplified_value),
                    position: position.clone(),
                });
                SimplifiedExpression {
                    content,
                    data_type: DataType::scalar(BaseType::Void),
                }
            }
        })
    }

    pub(super) fn simplify_assert_statement(
        &mut self,
        value: &i::Expression,
        position: &FilePosition,
    ) -> Result<SimplifiedExpression, CompileProblem> {
        let simplified_value = self.simplify_expression(value)?;
        Result::Ok(match simplified_value.content {
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
