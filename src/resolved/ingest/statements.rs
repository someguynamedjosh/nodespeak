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
            i::Statement::Branch { .. } => unimplemented!(),
            i::Statement::ForLoop { .. } => unimplemented!(),
            i::Statement::RawVPExpression(expr) => self.resolve_raw_vp_expression(expr),
        }
    }
}
