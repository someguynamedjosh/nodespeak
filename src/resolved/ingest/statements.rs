use super::{
    problems, util, DataType, ScopeSimplifier, SimplifiedStatement, SimplifiedVCExpression,
    SimplifiedVPExpression,
};
use crate::problem::{CompileProblem, FilePosition};
use crate::resolved::structure as o;
use crate::vague::structure as i;

impl<'a> ScopeSimplifier<'a> {
    fn resolve_creation_point(
        &mut self,
        old_var_id: i::VariableId,
        dtype: &i::VPExpression,
        position: &FilePosition,
    ) -> Result<SimplifiedStatement, CompileProblem> {
        let resolved_dtype = self.resolve_vp_expression(dtype)?;
        let data_type = if let SimplifiedVPExpression::Interpreted(data, ..) = resolved_dtype {
            if let i::KnownData::DataType(in_type) = data {
                self.input_to_intermediate_type(in_type)
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

        Ok(SimplifiedStatement::Interpreted)
    }

    fn resolve_assign_statement(
        &mut self,
        target: &i::VCExpression,
        value: &i::VPExpression,
        position: &FilePosition,
    ) -> Result<SimplifiedStatement, CompileProblem> {
        let lhs = self.resolve_vc_expression(target)?;
        let rhs = self.resolve_vp_expression(value)?;
        if &lhs.dtype != rhs.borrow_data_type() {
            // TODO: Better logic for when this applies.
            panic!("TODO: Nice error, bad assignment.");
        }
        Ok(SimplifiedStatement::Modified(o::Statement::Assign {
            target: Box::new(lhs.expr),
            value: Box::new(rhs.as_vp_expression()?),
            position: position.clone(),
        }))
    }

    fn resolve_raw_vp_expression(
        &mut self,
        expr: &i::VPExpression,
    ) -> Result<SimplifiedStatement, CompileProblem> {
        let resolved_expr = self.resolve_vp_expression(expr)?;
        if resolved_expr.borrow_data_type() != &DataType::Void {
            panic!("TODO: Nice error, vpe as statement yields an unused value.");
        }
        match resolved_expr {
            SimplifiedVPExpression::Interpreted(..) => Ok(SimplifiedStatement::Interpreted),
            SimplifiedVPExpression::Modified(new_expr, ..) => Ok(SimplifiedStatement::Modified(
                o::Statement::RawVPExpression(Box::new(new_expr)),
            )),
        }
    }

    pub(super) fn resolve_statement(
        &mut self,
        statement: &i::Statement,
    ) -> Result<SimplifiedStatement, CompileProblem> {
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
