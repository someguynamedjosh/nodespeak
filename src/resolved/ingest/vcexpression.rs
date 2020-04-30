use super::{
    problems, util, DataType, ScopeSimplifier, SimplifiedVCExpression, SimplifiedVPExpression,
};
use crate::problem::{CompileProblem, FilePosition};
use crate::resolved::structure as o;
use crate::vague::structure as i;
use std::borrow::Borrow;

impl<'a> ScopeSimplifier<'a> {
    pub(super) fn resolve_vc_variable(
        &mut self,
        var_id: i::VariableId,
        position: &FilePosition,
    ) -> Result<SimplifiedVCExpression, CompileProblem> {
        let (var_id, var_type) = self
            .get_var_info(var_id)
            .expect("Variable used before declaration, vague step should have caught this.");
        let var_id = var_id.expect("TODO: Nice error, variable not available at run time.");
        Ok(SimplifiedVCExpression {
            expr: o::VCExpression::Variable(var_id, position.clone()),
            dtype: var_type.clone(),
        })
    }

    pub(super) fn resolve_vc_expression(
        &mut self,
        input: &i::VCExpression,
    ) -> Result<SimplifiedVCExpression, CompileProblem> {
        match input {
            i::VCExpression::Variable(id, position) => self.resolve_vc_variable(*id, position),
            i::VCExpression::Index{..} => unimplemented!(),
        }
    }
}
