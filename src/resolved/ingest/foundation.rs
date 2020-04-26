use std::collections::HashMap;

use super::{
    util, BaseType, Content, DataType, ScopeSimplifier, SimplifiedExpression, SimplifierTable,
};
use crate::problem::{CompileProblem, FilePosition};
use crate::resolved::structure as o;
use crate::vague::structure as i;

impl<'a> ScopeSimplifier<'a> {
    pub(super) fn new(source: &'a mut i::Program) -> ScopeSimplifier<'a> {
        let target = o::Program::new();
        let entry_point = target.get_entry_point();
        ScopeSimplifier {
            source,
            target,
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

    pub(super) fn add_unresolved_auto_var(&mut self, var: i::VariableId) {
        self.table.unresolved_auto_vars.insert(var);
    }

    pub(super) fn is_unresolved_auto_var(&mut self, var: i::VariableId) -> bool {
        self.table.unresolved_auto_vars.contains(&var)
    }

    pub(super) fn mark_auto_var_resolved(&mut self, var: i::VariableId) {
        self.table.unresolved_auto_vars.remove(&var);
    }

    pub(super) fn set_temporary_value(&mut self, var: i::VariableId, value: i::KnownData) {
        self.temp_values.insert(var, value);
    }

    fn commit_temporary_to_array(
        &mut self,
        indexes: Vec<usize>,
        var: i::VariableId,
        array_data: &[i::KnownData],
    ) -> Result<Vec<i::KnownData>, CompileProblem> {
        let mut unknown = Vec::with_capacity(array_data.len());
        for (index, element) in array_data.iter().enumerate() {
            let mut next_indexes = indexes.clone();
            next_indexes.push(index);
            if let i::KnownData::Array(vec) = element {
                unknown.push(i::KnownData::Array(self.commit_temporary_to_array(
                    next_indexes,
                    var,
                    &vec[..],
                )?));
            } else if element != &i::KnownData::Unknown {
                unknown.push(i::KnownData::Unknown);
                let resolved = util::resolve_known_data(element)
                    .expect("TODO: Nice error, data cannot be used at run time.");
                let target = self
                    .convert(var)
                    .expect("TODO: Nice error, cannot convert variable.")
                    .clone();
                let indexes = next_indexes
                    .iter()
                    .map(|index| {
                        o::Expression::Literal(
                            o::KnownData::Int(*index as i64),
                            FilePosition::placeholder(),
                        )
                    })
                    .collect();
                self.target[self.current_scope].add_expression(o::Expression::Assign {
                    target: Box::new(o::Expression::Access {
                        base: Box::new(target),
                        indexes,
                        position: FilePosition::placeholder(),
                    }),
                    value: Box::new(o::Expression::Literal(
                        resolved,
                        FilePosition::placeholder(),
                    )),
                    position: FilePosition::placeholder(),
                })
            } else {
                unknown.push(i::KnownData::Unknown);
            }
        }
        Ok(unknown)
    }

    // This is used e.g. before branches where the fate of a variable with a known value is
    // uncertain. It will add instructions to copy the currently known value to the variable at
    // run time and then forget the currently known value.
    pub(super) fn commit_temporary_value_before_uncertainty(
        &mut self,
        var: i::VariableId,
    ) -> Result<(), CompileProblem> {
        let temporary_value = self.borrow_temporary_value(var);
        if temporary_value == &i::KnownData::Unknown {
            // The data is already unknown, we don't need to commit anything.
            return Ok(());
        }
        if let i::KnownData::Array(arr) = temporary_value {
            let arr = arr.clone();
            let unknown = self.commit_temporary_to_array(Vec::new(), var, &arr[..])?;
            self.temp_values.insert(var, i::KnownData::Array(unknown));
            Ok(())
        } else {
            let resolved_data = util::resolve_known_data(self.borrow_temporary_value(var))
                .expect("TODO: Nice error, data cannot be used at run time.");
            let target = self
                .convert(var)
                .expect("TODO: Nice error, cannot convert variable.")
                .clone();
            self.target[self.current_scope].add_expression(o::Expression::Assign {
                target: Box::new(target),
                value: Box::new(o::Expression::Literal(
                    resolved_data,
                    FilePosition::placeholder(),
                )),
                position: FilePosition::placeholder(),
            });
            self.temp_values.insert(var, i::KnownData::Unknown);
            Ok(())
        }
    }

    pub(super) fn borrow_temporary_value(&self, var: i::VariableId) -> &i::KnownData {
        self.temp_values
            .get(&var)
            .unwrap_or_else(|| &i::KnownData::Unknown)
    }

    pub(super) fn borrow_temporary_value_mut(&mut self, var: i::VariableId) -> &mut i::KnownData {
        if !self.temp_values.contains_key(&var) {
            self.set_temporary_value(var, i::KnownData::Unknown);
        }
        self.temp_values
            .get_mut(&var)
            .expect("We already guaranteed a value would be present.")
    }

    pub(super) fn int_literal(value: i64, position: FilePosition) -> o::Expression {
        o::Expression::Literal(o::KnownData::Int(value), position)
    }

    pub(super) fn copy_scope(
        &mut self,
        _source: i::ScopeId,
        parent: Option<o::ScopeId>,
    ) -> Result<o::ScopeId, CompileProblem> {
        let copy = match parent {
            Option::Some(parent_id) => self.target.create_child_scope(parent_id),
            Option::None => self.target.create_scope(),
        };

        // Variables are only copied once their i::Expression::CreationPoint is hit.

        Result::Ok(copy)
    }

    pub(super) fn resolve_expression(
        &mut self,
        old_expression: &i::Expression,
    ) -> Result<SimplifiedExpression, CompileProblem> {
        Result::Ok(match old_expression {
            i::Expression::Literal(value, ..) => SimplifiedExpression {
                data_type: self.input_to_intermediate_type(
                    value
                        .get_data_type()
                        .expect("Literals always have a known data type."),
                )?,
                content: Content::Interpreted(value.clone()),
            },
            i::Expression::Variable(id, position) => {
                self.resolve_variable(*id, position.clone())?
            }
            i::Expression::Access {
                base,
                indexes,
                position,
            } => {
                // TODO: Optimize accessing values in big arrays so that we don't have to clone
                // the entire array to pick out one value.
                let base_position = base.clone_position();
                let resolved_base = self.resolve_expression(base)?;
                let mut resolved_indexes = Vec::with_capacity(indexes.len());
                for index in indexes {
                    resolved_indexes
                        .push((self.resolve_expression(index)?, index.clone_position()));
                }
                self.resolve_access_expression(
                    resolved_base,
                    base_position,
                    resolved_indexes,
                    position.clone(),
                )?
            }
            i::Expression::InlineReturn(..) => unimplemented!(),

            i::Expression::UnaryOperation(..) => unimplemented!(),
            i::Expression::BinaryOperation(operand_1, operator, operand_2, position) => {
                let operand_1_position = operand_1.clone_position();
                let resolved_operand_1 = self.resolve_expression(operand_1)?;
                let operand_2_position = operand_2.clone_position();
                let resolved_operand_2 = self.resolve_expression(operand_2)?;
                self.resolve_binary_expression(
                    resolved_operand_1,
                    operand_1_position,
                    *operator,
                    resolved_operand_2,
                    operand_2_position,
                    position.clone(),
                )?
            }

            i::Expression::Collect(values, ..) => {
                let mut resolved_values = Vec::with_capacity(values.len());
                let mut value_positions = Vec::with_capacity(values.len());
                let mut all_known = true;
                for value in values {
                    value_positions.push(value.clone_position());
                    let resolved_value = self.resolve_expression(value)?;
                    if let Content::Modified(..) = &resolved_value.content {
                        all_known = false;
                    }
                    resolved_values.push(resolved_value);
                }
                if all_known {
                    let mut data = Vec::with_capacity(resolved_values.len());
                    for value in resolved_values {
                        data.push(match value.content {
                            Content::Interpreted(data) => data,
                            _ => unreachable!(
                                "We previously checked that all the data was interpreted."
                            ),
                        });
                    }
                    let final_data = i::KnownData::Array(data);
                    // TODO: Check that data types are compatible with each other.
                    SimplifiedExpression {
                        data_type: self.input_to_intermediate_type(
                            final_data
                                .get_data_type()
                                .expect("Data cannot be unknown, we just built it."),
                        )?,
                        content: Content::Interpreted(final_data),
                    }
                } else {
                    let mut items = Vec::with_capacity(resolved_values.len());
                    // TODO: Error for zero-sized arrays.
                    let data_type = resolved_values[0].data_type.clone();
                    // Standard treatment, fully interpreted values become literal expressions.
                    for (value, value_position) in
                        resolved_values.into_iter().zip(value_positions.into_iter())
                    {
                        // TODO: Check that all the elements are the same type.
                        match value.content {
                            Content::Interpreted(value) => {
                                let resolved =
                                    util::resolve_known_data(&value).expect("TODO: nice error");
                                items.push(o::Expression::Literal(resolved, value_position));
                            }
                            Content::Modified(expression) => items.push(expression),
                        }
                    }
                    unimplemented!("TODO: Proxy stuff.")
                }
            }
            i::Expression::CreationPoint(old_var_id, position) => {
                self.resolve_creation_point(*old_var_id, position)?
            }

            i::Expression::Assert(value, position) => {
                self.resolve_assert_statement(value, position)?
            }
            i::Expression::Assign {
                target,
                value,
                position,
            } => self.resolve_assign_statement(target, value, position)?,
            i::Expression::Return(position) => {
                let content = Content::Modified(o::Expression::Return(position.clone()));
                SimplifiedExpression {
                    content,
                    data_type: DataType::scalar(BaseType::Void),
                }
            }
            i::Expression::Branch {
                clauses,
                else_clause,
                position,
            } => self.resolve_branch(clauses, else_clause, position)?,
            i::Expression::ForLoop {
                counter,
                start,
                end,
                body,
                position,
            } => self.resolve_for_loop(*counter, start, end, *body, position)?,

            i::Expression::FuncCall {
                function,
                inputs,
                outputs,
                position,
            } => self.resolve_func_call(function, inputs, outputs, position)?,
        })
    }

    pub(super) fn resolve_scope(
        &mut self,
        source: i::ScopeId,
        parent: Option<o::ScopeId>,
    ) -> Result<o::ScopeId, CompileProblem> {
        let old_current_scope = self.current_scope;
        self.current_scope = self.copy_scope(source, parent)?;
        let old_body = self.source[source].borrow_body().clone();
        for expression in old_body {
            let resolved = self.resolve_expression(&expression)?;
            match resolved.content {
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
