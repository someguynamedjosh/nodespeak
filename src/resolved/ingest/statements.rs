use super::{problems, Content, ScopeSimplifier, SimplifiedExpression};
use crate::problem::{CompileProblem, FilePosition};
use crate::vague::structure as i;
use crate::resolved::structure as o;

impl<'a> ScopeSimplifier<'a> {
    pub(super) fn simplify_creation_point(
        &mut self,
        old_var_id: i::VariableId,
        position: &FilePosition
    ) -> Result<SimplifiedExpression, CompileProblem> {
        let old_data_type = self.source[old_var_id].borrow_data_type();
        let old_base_type = old_data_type.borrow_base().clone();
        let old_dimensions = old_data_type.borrow_dimensions().clone();
        let mut new_dimensions = vec![];
        for old_dimension in old_dimensions {
            match self.simplify_expression(&old_dimension)?.content {
                Content::Interpreted(data) => match data {
                    i::KnownData::Int(value) => if value > 0 {
                        new_dimensions.push(value as u64);
                    } else {
                        panic!("TODO: Nice error, nonpositive array size.");
                    },
                    _ => panic!("TODO: Nice error, array size must be int.")
                }
                Content::Modified(..) => panic!("TODO: Nice error, array size not resolved at runtime.")
            }
        }
        match old_base_type {
            i::BaseType::Automatic => unimplemented!("TODO: Automatic types."),
            i::BaseType::Dynamic(..) => unimplemented!("TODO: Dynamic types."),
            i::BaseType::LoadTemplateParameter(..) => unimplemented!("TODO: Template parameters."),
            i::BaseType::Bool => {
                let new_data_type = o::DataType::array(o::BaseType::Bool, new_dimensions);
                let variable = o::Variable::new(position.clone(), new_data_type);
                let new_var_id = self.target.adopt_and_define_intermediate(self.current_scope, variable);
                self.add_conversion(old_var_id, new_var_id);
            }
            i::BaseType::Int => {
                let new_data_type = o::DataType::array(o::BaseType::Int, new_dimensions);
                let variable = o::Variable::new(position.clone(), new_data_type);
                let new_var_id = self.target.adopt_and_define_intermediate(self.current_scope, variable);
                self.add_conversion(old_var_id, new_var_id);
            }
            i::BaseType::Float => {
                let new_data_type = o::DataType::array(o::BaseType::Float, new_dimensions);
                let variable = o::Variable::new(position.clone(), new_data_type);
                let new_var_id = self.target.adopt_and_define_intermediate(self.current_scope, variable);
                self.add_conversion(old_var_id, new_var_id);
            }
            i::BaseType::DataType_
            | i::BaseType::Function_
            | i::BaseType::Void => ()
        }
        
        Result::Ok(SimplifiedExpression {
            content: Content::Interpreted(i::KnownData::Void),
            data_type: None,
        })
    }

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
