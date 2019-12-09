use super::{problems, Content, ScopeSimplifier, SimplifiedExpression};
use crate::problem::{CompileProblem, FilePosition};
use crate::vague::structure as i;
use crate::resolved::structure as o;

impl<'a> ScopeSimplifier<'a> {
    pub(super) fn simplify_assign_statement(
        &mut self,
        target: &i::Expression,
        value: &i::Expression,
        position: &FilePosition,
    ) -> Result<SimplifiedExpression, CompileProblem> {
        let simplified_target = self.simplify_assignment_access_expression(target)?;
        let simplified_value = self.simplify_expression(value)?;
        if simplified_target.1.is_automatic() {
            self.simplify_automatic_type(&simplified_target.0, &simplified_value.data_type)?;
        } else if !simplified_value
            .data_type
            .equivalent(&simplified_target.1, self.program)
        {
            panic!("TODO: nice error, mismatched data types in assignment.");
        }
        Result::Ok(match simplified_value.content {
            Content::Interpreted(known_value) => {
                let result = self.assign_value_to_expression(
                    &simplified_target.0,
                    known_value,
                    position.clone(),
                )?;
                if let Result::Err(simplified_expresion) = result {
                    SimplifiedExpression {
                        content: Content::Modified(simplified_expresion),
                        data_type: i::DataType::scalar(i::BaseType::Void),
                    }
                } else {
                    SimplifiedExpression {
                        content: Content::Interpreted(i::KnownData::Void),
                        data_type: i::DataType::scalar(i::BaseType::Void),
                    }
                }
            }
            Content::Modified(simplified_value) => {
                self.assign_unknown_to_expression(&simplified_target.0);
                let content = Content::Modified(o::Expression::Assign {
                    target: Box::new(simplified_target.0),
                    value: Box::new(simplified_value),
                    position: position.clone(),
                });
                SimplifiedExpression {
                    content,
                    data_type: i::DataType::scalar(i::BaseType::Void),
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
                            data_type: i::DataType::scalar(i::BaseType::Void),
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
                data_type: i::DataType::scalar(i::BaseType::Void),
            },
        })
    }
}
