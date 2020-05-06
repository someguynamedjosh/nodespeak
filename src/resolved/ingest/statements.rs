use super::{
    DataType, PossiblyKnownData, ResolvedStatement, ResolvedVCExpression, ResolvedVPExpression,
    ScopeResolver,
};
use crate::high_level::problem::{CompileProblem, FilePosition};
use crate::resolved::structure as o;
use crate::vague::structure as i;

impl<'a> ScopeResolver<'a> {
    fn resolve_creation_point(
        &mut self,
        old_var_id: i::VariableId,
        dtype: &i::VPExpression,
        position: &FilePosition,
    ) -> Result<ResolvedStatement, CompileProblem> {
        let resolved_dtype = self.resolve_vp_expression(dtype)?;
        let data_type = if let ResolvedVPExpression::Interpreted(data, ..) = resolved_dtype {
            if let i::KnownData::DataType(in_type) = data {
                self.resolve_data_type_partially(in_type)
            } else {
                panic!("TODO: Nice error, value is not a data type")
            }
        } else {
            panic!("TODO: Nice error, expression was not computable at compile time")
        };
        let resolved_id = if let Some(data_type) = data_type.resolve() {
            let resolved_var = o::Variable::new(position.clone(), data_type);
            Some(self.target.adopt_variable(resolved_var))
        } else {
            None
        };
        self.set_var_info(old_var_id, resolved_id, data_type);
        if let Some(data) = self.source[old_var_id].borrow_initial_value() {
            let pkd = PossiblyKnownData::from_known_data(data);
            self.set_temporary_value(old_var_id, pkd);
        }

        Ok(ResolvedStatement::Interpreted)
    }

    pub(super) fn resolve_assign_statement(
        &mut self,
        target: &i::VCExpression,
        value: &i::VPExpression,
        position: &FilePosition,
    ) -> Result<ResolvedStatement, CompileProblem> {
        let lhs = self.resolve_vc_expression(target)?;
        let rhs = self.resolve_vp_expression(value)?;
        let mut resolved_out_type = None;
        if lhs.borrow_data_type().is_automatic() {
            let old_auto_type = &self.get_var_info(lhs.get_base()).unwrap().1;
            let actual_type = old_auto_type.with_different_base(rhs.borrow_data_type().clone());
            let resolved_var = if let Some(rtype) = actual_type.resolve() {
                resolved_out_type = Some(rtype.clone());
                let def_pos = self.source[lhs.get_base()].get_definition().clone();
                let var = o::Variable::new(def_pos, rtype);
                Some(self.target.adopt_variable(var))
            } else {
                None
            };
            self.resolve_auto_var(lhs.get_base(), resolved_var, actual_type);
        } else {
            resolved_out_type = lhs.borrow_data_type().resolve();
            let ok = match Self::biggest_type(lhs.borrow_data_type(), rhs.borrow_data_type()) {
                Ok(bct) => &bct == lhs.borrow_data_type(),
                Err(..) => false,
            };
            if !ok {
                panic!("TODO: Nice error, bad assignment.");
            }
        }
        if let (
            ResolvedVCExpression::Specific { var, indexes, .. },
            ResolvedVPExpression::Interpreted(value_data, ..),
        ) = (&lhs, &rhs)
        {
            self.set_temporary_item(
                *var,
                indexes,
                PossiblyKnownData::from_known_data(value_data),
            );
        } else {
            match &lhs {
                ResolvedVCExpression::Specific { var, indexes, .. }
                | ResolvedVCExpression::Modified {
                    base: var, indexes, ..
                } => {
                    self.reset_temporary_range(*var, indexes);
                }
            }
        }
        if resolved_out_type.is_some() {
            // Always return a modified expression. This way, even if we really know what the result
            // of the expression is at compile time, we can still ensure that if something about
            // this does end up being needed specifically at run time then it will be there.
            Ok(ResolvedStatement::Modified(o::Statement::Assign {
                target: Box::new(lhs.as_vc_expression(self)?),
                value: Box::new(rhs.as_vp_expression()?),
                position: position.clone(),
            }))
        } else {
            Ok(ResolvedStatement::Interpreted)
        }
    }

    // Use for clauses that might be executed but we don't know.
    fn resolve_clause_body(&mut self, body: i::ScopeId) -> Result<o::ScopeId, CompileProblem> {
        let scope = self.target.create_scope();
        let old_scope = self.current_scope;
        self.current_scope = scope;
        self.enter_branch_body();

        for statement in self.source[body].borrow_body().clone() {
            let res = self.resolve_statement(&statement)?;
            if let ResolvedStatement::Modified(rstatement) = res {
                self.target[self.current_scope].add_statement(rstatement);
            }
        }

        self.exit_branch_body();
        self.current_scope = old_scope;
        Ok(scope)
    }

    // Use for clauses that we are certain will be executed.
    fn resolve_clause_body_in_place(&mut self, body: i::ScopeId) -> Result<(), CompileProblem> {
        for statement in self.source[body].borrow_body().clone() {
            let res = self.resolve_statement(&statement)?;
            if let ResolvedStatement::Modified(rstatement) = res {
                self.target[self.current_scope].add_statement(rstatement);
            }
        }
        Ok(())
    }

    fn resolve_branch(
        &mut self,
        clauses: &Vec<(i::VPExpression, i::ScopeId)>,
        else_clause: &Option<i::ScopeId>,
        position: &FilePosition,
    ) -> Result<ResolvedStatement, CompileProblem> {
        let mut rclauses = Vec::new();
        for (condition, body) in clauses {
            let rcond = self.resolve_vp_expression(condition)?;
            if rcond.borrow_data_type() != &DataType::Bool {
                panic!("TODO: Nice error, condition is not boolean.");
            }
            if let ResolvedVPExpression::Interpreted(value, ..) = &rcond {
                // This is safe because we just checked that it is a bool.
                let value = value.require_bool();
                if value && rclauses.len() == 0 {
                    // If this clause is guaranteed to happen and no clauses could have happened
                    // before it, then this is the only clause that will execute. Resolve it in
                    // place and don't bother with the other clauses after it.
                    self.resolve_clause_body_in_place(*body)?;
                    return Ok(ResolvedStatement::Interpreted);
                } else if !value {
                    // If we know the clause won't be executed, just skip it.
                    continue;
                }
            }
            // We get here if we don't know what the condition is, or if we know what the condition
            // is but we can't do any optimizations about it.
            let rbody = self.resolve_clause_body(*body)?;
            rclauses.push((rcond.as_vp_expression()?, rbody));
        }
        let else_clause = if let Some(body) = else_clause {
            if rclauses.len() == 0 {
                // If no clauses could have been executed before the else clause, then we know that
                // the else clause must be executed, so just resolve it in place and return.
                self.resolve_clause_body_in_place(*body)?;
                return Ok(ResolvedStatement::Interpreted);
            } else {
                Some(self.resolve_clause_body(*body)?)
            }
        } else {
            None
        };
        Ok(ResolvedStatement::Modified(o::Statement::Branch {
            clauses: rclauses,
            else_clause,
            position: position.clone(),
        }))
    }

    fn resolve_for_loop(
        &mut self,
        counter: i::VariableId,
        start: &i::VPExpression,
        end: &i::VPExpression,
        body: i::ScopeId,
        position: &FilePosition,
    ) -> Result<ResolvedStatement, CompileProblem> {
        // Consider this:
        // Int val = 123; for i = 0 to 10 { other = val; val = val + i; }
        // If we just resolved the scope once, we would write "other = val" because val is known to
        // be 123 at that point. But since it is assigned to later on, we don't actually know that
        // val will be 123. But since it happens after we have already resolved the previous
        // statement, we can't retroactively change it. So instead, we resolve everything in the
        // for loop once, using enter_branch_body() at the start. Once that is complete,
        // exit_branch_body() will mark any variables that could have possibly been assigned as
        // Unknown. We can then go in and resolve the loop body for a second time which will yield
        // the correct code. This second resolving will not have any side effects because any
        // modified known values will be set back to unknown by exit_branch_body(), so known values
        // are either unchanged or invalidated by the first resolving and never actually changed.

        let counter_pos = self.source[counter].get_definition().clone();
        let rcounter = o::Variable::new(counter_pos, o::DataType::Int);
        let rcounter = self.target.adopt_variable(rcounter);
        self.set_var_info(counter, Some(rcounter), DataType::Int);
        let body = self.source[body].borrow_body().clone();
        let old_scope = self.current_scope;
        let rstart = self.resolve_vp_expression(start)?;
        let rend = self.resolve_vp_expression(end)?;
        if rstart.borrow_data_type() != &DataType::Int {
            panic!("TODO: Nice error, loop start not integer.");
        }
        if rend.borrow_data_type() != &DataType::Int {
            panic!("TODO: Nice error, loop end not integer.");
        }
        if let (
            ResolvedVPExpression::Interpreted(start, ..),
            ResolvedVPExpression::Interpreted(end, ..),
        ) = (&rstart, &rend)
        {
            // We just checked that they're ints.
            let start = start.require_int();
            let end = end.require_int();
            for i in start..end {
                self.set_temporary_value(counter, PossiblyKnownData::Int(i));
                for statement in &body {
                    let res = self.resolve_statement(statement)?;
                    if let ResolvedStatement::Modified(rstatement) = res {
                        self.target[self.current_scope].add_statement(rstatement);
                    }
                }
            }
            return Ok(ResolvedStatement::Interpreted);
        }

        let throwaway_scope = self.target.create_scope();
        self.current_scope = throwaway_scope;
        self.enter_branch_body();
        for statement in &body {
            // Don't bother adding it to the scope, it's junk code.
            self.resolve_statement(statement)?;
        }
        self.exit_branch_body();

        let real_scope = self.target.create_scope();
        self.current_scope = real_scope;
        // No enter branch body this time. Everything that gets assigned to with a fixed value will
        // legitimately have that value by the end of the loop. Since the previous section of code
        // just marked everything assigned during the loop as unknown, any known value as a result
        // of this next loop is the product of data that does not depend on the state of the loop.
        for statement in &body {
            let res = self.resolve_statement(statement)?;
            if let ResolvedStatement::Modified(rstatement) = res {
                self.target[self.current_scope].add_statement(rstatement);
            }
        }
        self.current_scope = old_scope;

        Ok(ResolvedStatement::Modified(o::Statement::ForLoop {
            counter: rcounter,
            start: Box::new(rstart.as_vp_expression()?),
            end: Box::new(rend.as_vp_expression()?),
            body: real_scope,
            position: position.clone(),
        }))
    }

    fn resolve_raw_vp_expression(
        &mut self,
        expr: &i::VPExpression,
    ) -> Result<ResolvedStatement, CompileProblem> {
        let resolved_expr = self.resolve_vp_expression(expr)?;
        if let ResolvedVPExpression::Interpreted(data, ..) = resolved_expr {
            if data != i::KnownData::Void {
                panic!("TODO: Nice error, vpe as statement yields an unused value.");
            } else {
                Ok(ResolvedStatement::Interpreted)
            }
        } else {
            panic!("TODO: Nice error, vpe as statement yields an unused value.");
        }
    }

    pub(super) fn resolve_statement(
        &mut self,
        statement: &i::Statement,
    ) -> Result<ResolvedStatement, CompileProblem> {
        match statement {
            i::Statement::CreationPoint {
                var,
                var_type,
                position,
            } => self.resolve_creation_point(*var, var_type, position),
            i::Statement::Assert(..) => unimplemented!(),
            i::Statement::Return(..) => unimplemented!(),
            i::Statement::Assign {
                target,
                value,
                position,
            } => self.resolve_assign_statement(target, value, position),
            i::Statement::Branch {
                clauses,
                else_clause,
                position,
            } => self.resolve_branch(clauses, else_clause, position),
            i::Statement::ForLoop {
                counter,
                start,
                end,
                body,
                position,
            } => self.resolve_for_loop(*counter, start, end, *body, position),
            i::Statement::RawVPExpression(expr) => self.resolve_raw_vp_expression(expr),
        }
    }
}
