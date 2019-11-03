use super::{problems, Content, ScopeSimplifier, SimplifiedExpression};
use crate::problem::{CompileProblem, FilePosition};
use crate::util::NVec;
use crate::vague::structure::{
    BaseType, DataType, Expression, KnownData, Program, ScopeId, VariableId,
};
use std::collections::HashMap;

impl<'a> ScopeSimplifier<'a> {
    pub(super) fn new(program: &'a mut Program) -> ScopeSimplifier<'a> {
        ScopeSimplifier {
            program,
            conversion_table: HashMap::new(),
            conversion_stack: Vec::new(),
        }
    }

    // Pushes the current state of the conversion table onto the stack. The state
    // can be restored with pop_table().
    pub(super) fn push_table(&mut self) {
        self.conversion_stack.push(self.conversion_table.clone());
    }

    pub(super) fn pop_table(&mut self) {
        self.conversion_table = self
            .conversion_stack
            .pop()
            .expect("Encountered extra unexpected stack pop");
    }

    pub(super) fn add_conversion(&mut self, from: VariableId, to: VariableId) {
        assert!(
            !self.conversion_table.contains_key(&from),
            "Cannot have multiple conversions for a single variable."
        );
        self.conversion_table.insert(from, to);
    }

    pub(super) fn convert(&self, from: VariableId) -> VariableId {
        // Either the ID was remapped to something else, or the ID has remained
        // unchanged.
        *self.conversion_table.get(&from).unwrap_or(&from)
    }

    pub(super) fn int_literal(value: i64, position: FilePosition) -> Expression {
        Expression::Literal(KnownData::Int(value), position)
    }

    pub(super) fn copy_scope(
        &mut self,
        source: ScopeId,
        parent: Option<ScopeId>,
    ) -> Result<ScopeId, CompileProblem> {
        let copy = match parent {
            Option::Some(parent_id) => self.program.create_child_scope(parent_id),
            Option::None => self.program.create_scope(),
        };

        // TODO: We probably don't need to preserve the variable names in the
        // simplified scope. Depends on how some meta features get implemented in
        // the future.
        let symbol_table = self.program[source].borrow_symbols().clone();
        for name_value_pair in symbol_table.iter() {
            let old = *name_value_pair.1;
            let variable = self.program[old].clone();
            // TODO: we might not need to clone every variable.
            let new = self
                .program
                .adopt_and_define_symbol(copy, name_value_pair.0, variable);
            self.add_conversion(old, new)
        }

        let intermediate_list = self.program[source].borrow_intermediates().clone();
        for old in intermediate_list.into_iter() {
            let variable = self.program[old].clone();
            // TODO: we might not need to clone every variable.
            let new = self.program.adopt_and_define_intermediate(copy, variable);
            self.add_conversion(old, new)
        }

        for old_input in self.program[source].borrow_inputs().clone() {
            let converted = self.convert(old_input);
            self.program[copy].add_input(converted);
        }

        for old_output in self.program[source].borrow_outputs().clone() {
            let converted = self.convert(old_output);
            self.program[copy].add_output(converted);
        }

        Result::Ok(copy)
    }

    pub(super) fn simplify_expression(
        &mut self,
        old_expression: &Expression,
    ) -> Result<SimplifiedExpression, CompileProblem> {
        Result::Ok(match old_expression {
            Expression::Literal(value, ..) => SimplifiedExpression {
                data_type: value
                    .get_data_type()
                    .expect("Literals always have a known data type."),
                content: Content::Interpreted(value.clone()),
            },
            Expression::Variable(id, position) => {
                let converted_id = self.convert(*id);
                let temporary_value = self.program[converted_id].borrow_temporary_value();
                match temporary_value {
                    KnownData::Unknown => {
                        let content =
                            Content::Modified(Expression::Variable(converted_id, position.clone()));
                        let data_type = self.program[converted_id].borrow_data_type().clone();
                        SimplifiedExpression { content, data_type }
                    }
                    _ => {
                        let content = Content::Interpreted(temporary_value.clone());
                        let data_type = temporary_value
                            .get_data_type()
                            .expect("Already checked that data was not uknown.");
                        SimplifiedExpression { content, data_type }
                    }
                }
            }
            Expression::Proxy { .. } => unimplemented!(),
            Expression::Access {
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
            Expression::InlineReturn(..) => unimplemented!(),

            Expression::UnaryOperation(..) => unimplemented!(),
            Expression::BinaryOperation(operand_1, operator, operand_2, position) => {
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

            Expression::AssignInput{..} => unreachable!("Should be handled elsewhere."),
            Expression::AssignOutput{..} => unreachable!("Should be handled elsewhere."),
            Expression::Collect(values, position) => {
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
                    if let KnownData::Array(..) = &data[0] {
                        // Get all the sub arrays and collect them into a higher-dimension array.
                        let mut sub_arrays = Vec::with_capacity(data.len());
                        for datum in data {
                            sub_arrays.push(match datum {
                                KnownData::Array(sub_array) => sub_array,
                                _ => panic!("TODO: nice error, data types incompatible."), })
                        }
                        let final_data = KnownData::Array(NVec::collect(sub_arrays));
                        SimplifiedExpression {
                            data_type: final_data
                                .get_data_type()
                                .expect("Data cannot be unknown, we just built it."),
                            content: Content::Interpreted(final_data),
                        }
                    } else {
                        // Each element is a scalar, so just make a new 1d array out of them.
                        let final_data = KnownData::Array(NVec::from_vec(data));
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
                                items.push(Expression::Literal(value, value_position))
                            }
                            Content::Modified(expression) => items.push(expression),
                        }
                    }
                    SimplifiedExpression {
                        content: Content::Modified(Expression::Collect(items, position.clone())),
                        data_type,
                    }
                }
            }
            Expression::CreationPoint(old_var_id, ..) => {
                let new_var_id = self.convert(*old_var_id);
                let data_type = self.program[new_var_id].borrow_data_type();
                let mut new_sizes = Vec::with_capacity(data_type.borrow_dimensions().len());
                for size in data_type.borrow_dimensions().clone() {
                    let content = self.simplify_expression(&size)?.content;
                    new_sizes.push(content.into_expression(size.clone_position()));
                }
                self.program[new_var_id]
                    .borrow_data_type_mut()
                    .set_dimensions(new_sizes);
                SimplifiedExpression {
                    content: Content::Interpreted(KnownData::Void),
                    data_type: DataType::scalar(BaseType::Void),
                }
            }

            Expression::Assert(value, position) => {
                let simplified_value = self.simplify_expression(value)?;
                match simplified_value.content {
                    Content::Interpreted(value) => {
                        if let KnownData::Bool(succeed) = value {
                            if succeed {
                                SimplifiedExpression {
                                    content: Content::Interpreted(KnownData::Void),
                                    data_type: DataType::scalar(BaseType::Void),
                                }
                            } else {
                                panic!("TODO: nice error, assert guaranteed to fail.")
                            }
                        } else {
                            panic!("TODO: nice error, argument to assert is not bool.")
                        }
                    }
                    Content::Modified(expression) => SimplifiedExpression {
                        content: Content::Modified(Expression::Assert(
                            Box::new(expression),
                            position.clone(),
                        )),
                        data_type: DataType::scalar(BaseType::Void),
                    },
                }
            }
            Expression::Assign {
                target,
                value,
                position,
            } => self.simplify_assign_statement(target, value, position)?,
            Expression::Return(position) => {
                let content = Content::Modified(Expression::Return(position.clone()));
                SimplifiedExpression {
                    content,
                    data_type: DataType::scalar(BaseType::Void),
                }
            }

            Expression::FuncCall {
                function,
                setup,
                teardown,
                position,
            } => self.simplify_func_call(function, setup, teardown, position)?,
        })
    }

    pub(super) fn simplify_scope(&mut self, source: ScopeId) -> Result<ScopeId, CompileProblem> {
        let copied = self.copy_scope(source, self.program[source].get_parent())?;
        let old_body = self.program[source].borrow_body().clone();
        for expression in old_body {
            let simplified = self.simplify_expression(&expression)?;
            match simplified.content {
                // If it was successfully interpreted, we don't need to do anything.
                Content::Interpreted(..) => (),
                Content::Modified(expression) => match expression {
                    // TODO: Warn for expressions that have outputs that are not stored.
                    Expression::Return(..) => break,
                    _ => self.program[copied].add_expression(expression),
                },
            }
        }
        Result::Ok(copied)
    }
}
