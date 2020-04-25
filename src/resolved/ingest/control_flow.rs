use std::borrow::Borrow;
use std::collections::HashSet;

use super::{BaseType, CompileProblem, Content, DataType, ScopeSimplifier, SimplifiedExpression};
use crate::problem::FilePosition;
use crate::resolved::structure as o;
use crate::vague::structure as i;

impl<'a> ScopeSimplifier<'a> {
    pub(super) fn resolve_scope_in_place(
        &mut self,
        old_scope: i::ScopeId,
    ) -> Result<(), CompileProblem> {
        // TODO: Check if this causes any scoping weirdness.
        for expression in self.source[old_scope].borrow_body().clone() {
            if let Content::Modified(new) = self.resolve_expression(&expression)?.content {
                self.target[self.current_scope].add_expression(new);
            }
        }
        Ok(())
    }

    pub(super) fn resolve_scope_as_child(
        &mut self,
        old_scope: i::ScopeId,
    ) -> Result<o::ScopeId, CompileProblem> {
        let new_scope = self.copy_scope(old_scope, Some(self.current_scope))?;
        let old_current_scope = self.current_scope;
        self.current_scope = new_scope;
        for expression in self.source[old_scope].borrow_body().clone() {
            if let Content::Modified(new) = self.resolve_expression(&expression)?.content {
                self.target[self.current_scope].add_expression(new);
            }
        }
        self.current_scope = old_current_scope;
        Ok(new_scope)
    }

    pub(super) fn resolve_branch(
        &mut self,
        clauses: &Vec<(i::Expression, i::ScopeId)>,
        else_clause: &Option<i::ScopeId>,
        position: &FilePosition,
    ) -> Result<SimplifiedExpression, CompileProblem> {
        let mut new_clauses = Vec::new();
        let mut new_else_clause = None;
        let mut skip_else_clause = false;
        for (condition, body) in clauses {
            let resolved_condition = self.resolve_expression(condition)?;
            match resolved_condition.content {
                Content::Interpreted(i::KnownData::Bool(true)) => {
                    // Guaranteed to happen.
                    if new_clauses.len() == 0 {
                        // The first clause that could possibly happen is guaranteed to happen.
                        self.resolve_scope_in_place(*body)?;
                    } else {
                        // This clause is guaranteed to happen if none of the other ones happen.
                        let child_scope = self.resolve_scope_as_child(*body)?;
                        new_else_clause = Some(child_scope);
                    }
                    // Guaranteed that control will not pass on to further clauses.
                    skip_else_clause = true;
                    break;
                }
                // Guaranteed not to happen.
                Content::Interpreted(i::KnownData::Bool(false)) => (),
                // Condition must be a boolean.
                Content::Interpreted(..) => {
                    panic!("TODO: Nice error, condition of if is not a bool.")
                }
                Content::Modified(new_condition) => {
                    // We don't know if this branch will be executed, so we should commit all
                    // known variables.
                    let assigned_variables = self.collect_assignments_in_body_wrapper(*body);
                    for assigned_variable in assigned_variables.clone() {
                        self.commit_temporary_value_before_uncertainty(assigned_variable)?;
                    }
                    let new_scope = self.copy_scope(*body, Some(self.current_scope))?;
                    let old_current_scope = self.current_scope;
                    self.current_scope = new_scope;
                    for expression in self.source[*body].borrow_body().clone() {
                        if let Content::Modified(new) =
                            self.resolve_expression(&expression)?.content
                        {
                            self.target[self.current_scope].add_expression(new);
                        }
                    }
                    // Make sure that any values we know in the body of the if clause actually get
                    // assigned there.
                    for assigned_variable in assigned_variables {
                        self.commit_temporary_value_before_uncertainty(assigned_variable)?;
                    }
                    self.current_scope = old_current_scope;
                    new_clauses.push((new_condition, new_scope));
                }
            }
        }
        if !skip_else_clause {
            if let Some(old_else_clause) = else_clause {
                if new_clauses.len() == 0 {
                    self.resolve_scope_in_place(*old_else_clause)?;
                } else {
                    // We don't know if this branch will be executed, so we should commit all
                    // known variables.
                    let assigned_variables =
                        self.collect_assignments_in_body_wrapper(*old_else_clause);
                    for assigned_variable in assigned_variables {
                        self.commit_temporary_value_before_uncertainty(assigned_variable)?;
                    }
                    new_else_clause = Some(self.resolve_scope_as_child(*old_else_clause)?);
                }
            }
        }
        Ok(SimplifiedExpression {
            data_type: DataType::scalar(BaseType::Void),
            content: if new_clauses.len() == 0 {
                Content::Interpreted(i::KnownData::Void)
            } else {
                Content::Modified(o::Expression::Branch {
                    clauses: new_clauses,
                    else_clause: new_else_clause,
                    position: position.clone(),
                })
            },
        })
    }

    fn collect_assignments_in_body(&self, body: i::ScopeId, output: &mut HashSet<i::VariableId>) {
        for expr in self.source[body].borrow_body() {
            if let i::Expression::Assign { target, .. } = expr {
                if let i::Expression::Variable(id, ..) = target.borrow() {
                    output.insert(*id);
                } else if let i::Expression::Access { base, .. } = target.borrow() {
                    if let i::Expression::Variable(id, ..) = base.borrow() {
                        output.insert(*id);
                    }
                } else {
                    unreachable!("Encountered invalid assignment expression.");
                }
            } else if let i::Expression::Branch {
                clauses,
                else_clause,
                ..
            } = expr
            {
                for (.., body) in clauses {
                    self.collect_assignments_in_body(*body, output);
                }
                if let Some(body) = else_clause {
                    self.collect_assignments_in_body(*body, output);
                }
            } else if let i::Expression::ForLoop { body, .. } = expr {
                self.collect_assignments_in_body(*body, output);
            } else if let i::Expression::FuncCall { outputs, .. } = expr {
                for func_output in outputs {
                    if let i::Expression::Variable(id, ..) = func_output {
                        output.insert(*id);
                    } else if let i::Expression::Access { base, .. } = func_output {
                        if let i::Expression::Variable(id, ..) = base.borrow() {
                            output.insert(*id);
                        }
                    } else {
                        unreachable!("Encountered invalid function output.");
                    }
                }
            }
        }
    }

    fn collect_assignments_in_body_wrapper(&self, body: i::ScopeId) -> HashSet<i::VariableId> {
        let mut hash_set = HashSet::new();
        self.collect_assignments_in_body(body, &mut hash_set);
        hash_set
    }

    pub(super) fn resolve_for_loop(
        &mut self,
        counter: i::VariableId,
        start: &i::Expression,
        end: &i::Expression,
        body: i::ScopeId,
        position: &FilePosition,
    ) -> Result<SimplifiedExpression, CompileProblem> {
        // TODO: Better metrics for loop unrolling.
        let resolved_start = self.resolve_expression(start)?;
        let resolved_end = self.resolve_expression(end)?;
        if resolved_start.data_type.borrow_base() != &BaseType::Int
            || resolved_start.data_type.is_array()
        {
            panic!("TODO: Nice error, for loop start must be integer.");
        }
        if resolved_end.data_type.borrow_base() != &BaseType::Int
            || resolved_end.data_type.is_array()
        {
            panic!("TODO: Nice error, for loop end must be integer.");
        }
        match (resolved_start.content, resolved_end.content) {
            (Content::Interpreted(known_start), Content::Interpreted(known_end)) => {
                // Unroll loop.
                // Requires OK because errors handled above.
                for counter_value in known_start.require_int()..known_end.require_int() {
                    self.set_temporary_value(counter, i::KnownData::Int(counter_value));
                    self.resolve_scope_in_place(body)?;
                }
                Ok(SimplifiedExpression {
                    content: Content::Interpreted(i::KnownData::Void),
                    data_type: DataType::scalar(BaseType::Void),
                })
            }
            (start_content, end_content) => {
                let assigned_variables = self.collect_assignments_in_body_wrapper(body);
                // Since they will be assigned multiple times during the loop body, we need to clear
                // their temporary values before we start resolving it otherwise other functions
                // will think that we know for sure what the content of these variables are even
                // though their values are written to inside the loop body.
                for assigned_variable in assigned_variables {
                    self.commit_temporary_value_before_uncertainty(assigned_variable)?;
                }
                let resolved_start = match start_content {
                    Content::Modified(new) => new,
                    Content::Interpreted(i::KnownData::Int(value)) => {
                        o::Expression::Literal(o::KnownData::Int(value), start.clone_position())
                    }
                    _ => unreachable!("Type errors handled above."),
                };
                let resolved_end = match end_content {
                    Content::Modified(new) => new,
                    Content::Interpreted(i::KnownData::Int(value)) => {
                        o::Expression::Literal(o::KnownData::Int(value), start.clone_position())
                    }
                    _ => unreachable!("Type errors handled above."),
                };
                // TODO: Better position for counter.
                let resolved_body = self.target.create_child_scope(self.current_scope);
                let resolved_counter_var =
                    o::Variable::new(position.clone(), o::DataType::scalar(o::BaseType::Int));
                let resolved_counter = self
                    .target
                    .adopt_and_define_intermediate(resolved_body, resolved_counter_var);
                self.target[resolved_body].add_input(resolved_counter);
                self.add_conversion(
                    counter,
                    o::Expression::Variable(resolved_counter, FilePosition::placeholder()),
                );
                let old_current_scope = self.current_scope;
                self.current_scope = resolved_body;
                for expression in self.source[body].borrow_body().clone() {
                    if let Content::Modified(new) = self.resolve_expression(&expression)?.content {
                        self.target[self.current_scope].add_expression(new);
                    }
                }
                self.current_scope = old_current_scope;
                Ok(SimplifiedExpression {
                    content: Content::Modified(o::Expression::ForLoop {
                        counter: resolved_counter,
                        start: Box::new(resolved_start),
                        end: Box::new(resolved_end),
                        body: resolved_body,
                        position: position.clone(),
                    }),
                    data_type: DataType::scalar(BaseType::Void),
                })
            }
        }
    }
}
