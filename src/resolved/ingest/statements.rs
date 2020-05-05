use super::{
    DataType, PossiblyKnownData, ResolvedStatement, ResolvedVCExpression, ResolvedVPExpression,
    ScopeResolver,
};
use crate::problem::{CompileProblem, FilePosition};
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
        let ok = match Self::biggest_type(lhs.borrow_data_type(), rhs.borrow_data_type()) {
            Ok(bct) => &bct == lhs.borrow_data_type(),
            Err(..) => false,
        };
        if !ok {
            panic!("TODO: Nice error, bad assignment.");
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
        if lhs.borrow_data_type().resolve().is_some() {
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
            rclauses.push(( rcond.as_vp_expression()?, rbody ));
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
            i::Statement::Branch { clauses, else_clause, position } => self.resolve_branch(clauses, else_clause, position),
            i::Statement::ForLoop { .. } => unimplemented!(),
            i::Statement::RawVPExpression(expr) => self.resolve_raw_vp_expression(expr),
        }
    }
}
