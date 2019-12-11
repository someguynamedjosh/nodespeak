use std::collections::HashMap;

use super::{Content, ScopeSimplifier, SimplifiedExpression, SimplifierTable};
use crate::problem::{CompileProblem, FilePosition};
use crate::resolved::structure as o;
use crate::util::NVec;
use crate::vague::structure as i;

impl<'a> ScopeSimplifier<'a> {
    pub(super) fn new(source: &'a mut i::Program) -> ScopeSimplifier<'a> {
        let entry_point = source.get_entry_point();
        ScopeSimplifier {
            source,
            target: o::Program::new(),
            current_scope: entry_point,
            table: SimplifierTable::new(),
            stack: Vec::new(),
            temp_values: HashMap::new(),
        }
    }

    // Pushes the current state of the conversion table onto the stack. The state
    // can be restored with pop_table().
    pub(super) fn push_table(&mut self) {
        self.stack.push(self.table.clone());
    }

    pub(super) fn pop_table(&mut self) {
        self.table = self
            .stack
            .pop()
            .expect("Encountered extra unexpected stack pop");
    }

    pub(super) fn add_conversion(&mut self, from: i::VariableId, to: o::Expression) {
        assert!(
            !self.table.conversions.contains_key(&from),
            "Cannot have multiple conversions for a single variable."
        );
        self.table.conversions.insert(from, to);
    }

    pub(super) fn convert(&self, from: i::VariableId) -> Option<&o::Expression> {
        self.table.conversions.get(&from)
    }

    pub(super) fn set_temporary_value(&self, var: i::VariableId, value: i::KnownData) {
        self.temp_values.insert(var, value);
    }

    pub(super) fn borrow_temporary_value(&self, var: i::VariableId) -> &i::KnownData {
        self.temp_values
            .get(&var)
            .unwrap_or_else(|| &i::KnownData::Void)
    }

    pub(super) fn int_literal(value: i64, position: FilePosition) -> o::Expression {
        unimplemented!()
    }

    pub(super) fn copy_scope(
        &mut self,
        source: i::ScopeId,
        parent: Option<o::ScopeId>,
    ) -> Result<o::ScopeId, CompileProblem> {
        let copy = match parent {
            Option::Some(parent_id) => self.target.create_child_scope(parent_id),
            Option::None => self.target.create_scope(),
        };

        // Variables are only copied once their i::Expression::CreationPoint is hit.

        Result::Ok(copy)
    }

    pub(super) fn simplify_expression(
        &mut self,
        old_expression: &i::Expression,
    ) -> Result<SimplifiedExpression, CompileProblem> {
        Result::Ok(match old_expression {
            i::Expression::Literal(value, ..) => SimplifiedExpression {
                data_type: value
                    .get_data_type()
                    .expect("Literals always have a known data type."),
                content: Content::Interpreted(value.clone()),
            },
            i::Expression::Variable(id, position) => {
                self.simplify_variable(*id, position.clone())?
            }
            i::Expression::Proxy { .. } => unimplemented!(),
            i::Expression::Access {
                base,
                indexes,
                position,
            } => {
                // TODO: Optimize accessing values in big arrays so that we don't have to clone
                // the entire array to pick out one value.
                let base_position = base.clone_position();
                let simplified_base = self.simplify_expression(base)?;
                let mut simplified_indexes = Vec::with_capacity(indexes.len());
                for index in indexes {
                    simplified_indexes
                        .push((self.simplify_expression(index)?, index.clone_position()));
                }
                self.simplify_access_expression(
                    simplified_base,
                    base_position,
                    simplified_indexes,
                    position.clone(),
                )?
            }
            i::Expression::InlineReturn(..) => unimplemented!(),

            i::Expression::UnaryOperation(..) => unimplemented!(),
            i::Expression::BinaryOperation(operand_1, operator, operand_2, position) => {
                let operand_1_position = operand_1.clone_position();
                let simplified_operand_1 = self.simplify_expression(operand_1)?;
                let operand_2_position = operand_2.clone_position();
                let simplified_operand_2 = self.simplify_expression(operand_2)?;
                self.simplify_binary_expression(
                    simplified_operand_1,
                    operand_1_position,
                    *operator,
                    simplified_operand_2,
                    operand_2_position,
                    position.clone(),
                )?
            }

            i::Expression::Collect(values, position) => {
                let mut simplified_values = Vec::with_capacity(values.len());
                let mut value_positions = Vec::with_capacity(values.len());
                let mut all_known = true;
                for value in values {
                    value_positions.push(value.clone_position());
                    let simplified_value = self.simplify_expression(value)?;
                    if let Content::Modified(..) = &simplified_value.content {
                        all_known = false;
                    }
                    simplified_values.push(simplified_value);
                }
                if all_known {
                    let mut data = Vec::with_capacity(simplified_values.len());
                    for value in simplified_values {
                        data.push(match value.content {
                            Content::Interpreted(data) => data,
                            _ => unreachable!(
                                "We previously checked that all the data was interpreted."
                            ),
                        });
                    }
                    // TODO: Check that data types are compatible with each other.
                    if let i::KnownData::Array(..) = &data[0] {
                        // Get all the sub arrays and collect them into a higher-dimension array.
                        let mut sub_arrays = Vec::with_capacity(data.len());
                        for datum in data {
                            sub_arrays.push(match datum {
                                i::KnownData::Array(sub_array) => sub_array,
                                _ => panic!("TODO: nice error, data types incompatible."),
                            })
                        }
                        let final_data = i::KnownData::Array(NVec::collect(sub_arrays));
                        SimplifiedExpression {
                            data_type: final_data
                                .get_data_type()
                                .expect("Data cannot be unknown, we just built it."),
                            content: Content::Interpreted(final_data),
                        }
                    } else {
                        // Each element is a scalar, so just make a new 1d array out of them.
                        let final_data = i::KnownData::Array(NVec::from_vec(data));
                        SimplifiedExpression {
                            data_type: final_data
                                .get_data_type()
                                .expect("Data cannot be unknown, we just built it."),
                            content: Content::Interpreted(final_data),
                        }
                    }
                } else {
                    let mut items = Vec::with_capacity(simplified_values.len());
                    // TODO: Error for zero-sized arrays.
                    let data_type = simplified_values[0].data_type.clone();
                    // Standard treatment, fully interpreted values become literal expressions.
                    for (value, value_position) in simplified_values
                        .into_iter()
                        .zip(value_positions.into_iter())
                    {
                        // TODO: Check that all the elements are the same type.
                        match value.content {
                            Content::Interpreted(value) => {
                                items.push(i::Expression::Literal(value, value_position))
                            }
                            Content::Modified(expression) => items.push(expression),
                        }
                    }
                    SimplifiedExpression {
                        content: Content::Modified(i::Expression::Collect(items, position.clone())),
                        data_type,
                    }
                }
            }
            i::Expression::CreationPoint(old_var_id, position) => {
                self.simplify_creation_point(*old_var_id, position)?;
                let new_var_id = self.convert(*old_var_id);
                let data_type = self.target[new_var_id].borrow_data_type();
                let mut new_sizes = Vec::with_capacity(data_type.borrow_dimensions().len());
                for size in data_type.borrow_dimensions().clone() {
                    let content = self.simplify_expression(&size)?.content;
                    new_sizes.push(content.into_expression(size.clone_position()));
                }
                self.target[new_var_id]
                    .borrow_data_type_mut()
                    .set_dimensions(new_sizes);
                SimplifiedExpression {
                    content: Content::Interpreted(i::KnownData::Void),
                    data_type: DataType::scalar(BaseType::Void),
                }
            }

            i::Expression::Assert(value, position) => {
                self.simplify_assert_statement(value, position)?
            }
            i::Expression::Assign {
                target,
                value,
                position,
            } => self.simplify_assign_statement(target, value, position)?,
            i::Expression::Return(position) => {
                let content = Content::Modified(i::Expression::Return(position.clone()));
                SimplifiedExpression {
                    content,
                    data_type: DataType::scalar(BaseType::Void),
                }
            }

            i::Expression::FuncCall {
                function,
                inputs,
                outputs,
                position,
            } => self.simplify_func_call(function, inputs, outputs, position)?,
        })
    }

    pub(super) fn simplify_scope(
        &mut self,
        source: i::ScopeId,
        parent: Option<o::ScopeId>,
    ) -> Result<o::ScopeId, CompileProblem> {
        let old_current_scope = self.current_scope;
        self.current_scope = self.copy_scope(source, parent)?;
        let old_body = self.source[source].borrow_body().clone();
        for expression in old_body {
            let simplified = self.simplify_expression(&expression)?;
            match simplified.content {
                // If it was successfully interpreted, we don't need to do anything.
                Content::Interpreted(..) => (),
                Content::Modified(expression) => match expression {
                    // TODO: Warn for expressions that have outputs that are not stored.
                    o::Expression::Return(..) => break,
                    _ => self.target[self.current_scope].add_expression(expression),
                },
            }
        }
        let result = Result::Ok(self.current_scope);
        self.current_scope = old_current_scope;
        result
    }
}
